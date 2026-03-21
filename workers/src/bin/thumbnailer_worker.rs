use anyhow::Context;
use redis::aio::MultiplexedConnection;
use sqlx::FromRow;
use sqlx::postgres::PgPoolOptions;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use tokio::process::Command;
use uuid::Uuid;

#[path = "../logging.rs"]
mod logging;

const STATUS_READY: i16 = 1;
const STATUS_FAILED: i16 = 2;
const STATUS_THUMBNAILING: i16 = 3;

const MEDIA_TYPE_IMAGE: i16 = 0;
const MEDIA_TYPE_VIDEO: i16 = 1;
const MEDIA_TYPE_AUDIO: i16 = 2;

#[derive(Debug, FromRow)]
struct FileRow {
    id: Uuid,
    path: String,
    media_type: i16,
    status: i16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    logging::init_logger().context("failed to initialize logger")?;

    let db_url = std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?;
    let redis_url = std::env::var("REDIS_URL").context("REDIS_URL is not set")?;
    let thumbnailer_queue =
        std::env::var("THUMBNAILER_QUEUE").unwrap_or_else(|_| "thumbnailer:jobs".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("failed to connect to postgres")?;

    log::info!("thumbnailer worker started; queue={thumbnailer_queue}");

    loop {
        let mut redis_conn = match connect_redis(&redis_url).await {
            Ok(conn) => conn,
            Err(err) => {
                log::error!("failed to connect redis: {err}");
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        loop {
            let (_, payload): (String, String) = match redis::cmd("BLPOP")
                .arg(&thumbnailer_queue)
                .arg(0)
                .query_async(&mut redis_conn)
                .await
            {
                Ok(data) => data,
                Err(err) => {
                    log::error!("redis BLPOP failed: {err}");
                    break;
                }
            };

            let file_id = match Uuid::from_str(&payload) {
                Ok(id) => id,
                Err(err) => {
                    log::warn!("invalid file id in payload {payload}: {err}");
                    continue;
                }
            };

            if let Err(err) = process_file(&pool, file_id).await {
                log::error!("thumbnailing failed for {file_id}: {err}");
            }
        }
    }
}

async fn connect_redis(url: &str) -> anyhow::Result<MultiplexedConnection> {
    let client = redis::Client::open(url).context("invalid REDIS_URL")?;
    let conn = client
        .get_multiplexed_async_connection()
        .await
        .context("unable to establish redis connection")?;
    Ok(conn)
}

async fn process_file(pool: &sqlx::PgPool, file_id: Uuid) -> anyhow::Result<()> {
    let row = sqlx::query_as::<_, FileRow>(
        "SELECT id, path, media_type, status FROM files WHERE id = $1",
    )
    .bind(file_id)
    .fetch_optional(pool)
    .await
    .with_context(|| format!("failed to fetch file row {file_id}"))?;

    let Some(file) = row else {
        log::warn!("file {file_id} was not found in db");
        return Ok(());
    };

    if file.status != STATUS_THUMBNAILING {
        log::debug!("skip file {}: status is {}", file.id, file.status);
        return Ok(());
    }

    let source_path = PathBuf::from(&file.path);
    let (small_thumb, large_thumb) = thumbnail_paths(file.id);
    ensure_parent_dirs(&small_thumb).await?;
    ensure_parent_dirs(&large_thumb).await?;

    let thumbs_result = match file.media_type {
        MEDIA_TYPE_IMAGE => render_image_thumbs(&source_path, &small_thumb, &large_thumb).await,
        MEDIA_TYPE_VIDEO => render_video_thumbs(&source_path, &small_thumb, &large_thumb).await,
        MEDIA_TYPE_AUDIO => render_audio_thumbs(&source_path, &small_thumb, &large_thumb).await,
        other => Err(anyhow::anyhow!("unsupported media_type {}", other)),
    };

    if let Err(err) = thumbs_result {
        log::warn!("thumbnailer failed for {}: {err}", file.id);
        mark_status(pool, file.id, STATUS_FAILED).await?;
        return Ok(());
    }

    mark_status(pool, file.id, STATUS_READY).await?;
    log::info!("thumbnails ready for {}", file.id);
    Ok(())
}

fn thumbnail_paths(file_id: Uuid) -> (PathBuf, PathBuf) {
    let id = file_id.to_string();
    let shard_a = &id[0..2];
    let shard_b = &id[2..4];

    let base = PathBuf::from("/media/new")
        .join("thumb")
        .join(shard_a)
        .join(shard_b);

    let small = base.join(format!("{}_small.webp", id));
    let large = base.join(format!("{}_large.webp", id));
    (small, large)
}

async fn ensure_parent_dirs(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Ok(())
}

async fn render_image_thumbs(
    source: &Path,
    small_out: &Path,
    large_out: &Path,
) -> anyhow::Result<()> {
    render_frame(
        source,
        small_out,
        "scale=w=480:h=480:force_original_aspect_ratio=decrease",
    )
    .await?;
    render_frame(
        source,
        large_out,
        "scale=w=1920:h=1080:force_original_aspect_ratio=decrease",
    )
    .await?;
    Ok(())
}

async fn render_video_thumbs(
    source: &Path,
    small_out: &Path,
    large_out: &Path,
) -> anyhow::Result<()> {
    render_video_frame(
        source,
        small_out,
        "scale=w=480:h=480:force_original_aspect_ratio=decrease",
    )
    .await?;
    render_video_frame(
        source,
        large_out,
        "scale=w=1920:h=1080:force_original_aspect_ratio=decrease",
    )
    .await?;
    Ok(())
}

async fn render_audio_thumbs(
    source: &Path,
    small_out: &Path,
    large_out: &Path,
) -> anyhow::Result<()> {
    if !has_video_stream(source).await? {
        log::info!(
            "audio has no cover art, skip thumbnails for {}",
            source.display()
        );
        return Ok(());
    }

    render_audio_cover(
        source,
        small_out,
        "scale=w=480:h=480:force_original_aspect_ratio=decrease",
    )
    .await?;
    render_audio_cover(
        source,
        large_out,
        "scale=w=1920:h=1080:force_original_aspect_ratio=decrease",
    )
    .await?;
    Ok(())
}

async fn render_frame(source: &Path, output: &Path, scale: &str) -> anyhow::Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(source)
        .arg("-frames:v")
        .arg("1")
        .arg("-vf")
        .arg(scale)
        .arg("-c:v")
        .arg("libwebp")
        .arg("-quality")
        .arg("80")
        .arg(output)
        .status()
        .await
        .with_context(|| format!("failed to execute ffmpeg for {}", source.display()))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "ffmpeg image thumbnail command failed for {}",
            source.display()
        ))
    }
}

async fn render_video_frame(source: &Path, output: &Path, scale: &str) -> anyhow::Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss")
        .arg("00:00:01")
        .arg("-i")
        .arg(source)
        .arg("-frames:v")
        .arg("1")
        .arg("-vf")
        .arg(scale)
        .arg("-c:v")
        .arg("libwebp")
        .arg("-quality")
        .arg("80")
        .arg(output)
        .status()
        .await
        .with_context(|| format!("failed to execute ffmpeg for {}", source.display()))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "ffmpeg video thumbnail command failed for {}",
            source.display()
        ))
    }
}

async fn render_audio_cover(source: &Path, output: &Path, scale: &str) -> anyhow::Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(source)
        .arg("-an")
        .arg("-frames:v")
        .arg("1")
        .arg("-vf")
        .arg(scale)
        .arg("-c:v")
        .arg("libwebp")
        .arg("-quality")
        .arg("80")
        .arg(output)
        .status()
        .await
        .with_context(|| format!("failed to execute ffmpeg for {}", source.display()))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "ffmpeg audio-cover thumbnail command failed for {}",
            source.display()
        ))
    }
}

async fn has_video_stream(source: &Path) -> anyhow::Result<bool> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=index")
        .arg("-of")
        .arg("csv=p=0")
        .arg(source)
        .output()
        .await
        .with_context(|| format!("failed to execute ffprobe for {}", source.display()))?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(!stdout.trim().is_empty())
}

async fn mark_status(pool: &sqlx::PgPool, file_id: Uuid, status: i16) -> anyhow::Result<()> {
    sqlx::query("UPDATE files SET status = $2 WHERE id = $1")
        .bind(file_id)
        .bind(status)
        .execute(pool)
        .await
        .with_context(|| format!("failed to set status {} for {}", status, file_id))?;
    Ok(())
}
