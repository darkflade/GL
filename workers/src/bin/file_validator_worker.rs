use anyhow::Context;
use infer::MatcherType;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use sqlx::FromRow;
use sqlx::postgres::PgPoolOptions;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

#[path = "../logging.rs"]
mod logging;

const STATUS_VALIDATING: i16 = 0;
const STATUS_FAILED: i16 = 2;
const STATUS_THUMBNAILING: i16 = 3;

#[derive(Debug, FromRow)]
struct FileRow {
    id: Uuid,
    path: String,
    status: i16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    logging::init_logger().context("failed to initialize logger")?;

    let db_url = std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?;
    let redis_url = std::env::var("REDIS_URL").context("REDIS_URL is not set")?;
    let validator_queue =
        std::env::var("FILE_VALIDATOR_QUEUE").unwrap_or_else(|_| "file_validator:jobs".to_string());
    let thumbnailer_queue =
        std::env::var("THUMBNAILER_QUEUE").unwrap_or_else(|_| "thumbnailer:jobs".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("failed to connect to postgres")?;

    log::info!(
        "file validator worker started; validator_queue={validator_queue}, thumbnailer_queue={thumbnailer_queue}"
    );

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
                .arg(&validator_queue)
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

            if let Err(err) =
                process_file(&pool, &mut redis_conn, &thumbnailer_queue, file_id).await
            {
                log::error!("validator failed for file {file_id}: {err}");
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

async fn process_file(
    pool: &sqlx::PgPool,
    redis_conn: &mut MultiplexedConnection,
    thumbnailer_queue: &str,
    file_id: Uuid,
) -> anyhow::Result<()> {
    let row = sqlx::query_as::<_, FileRow>("SELECT id, path, status FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("failed to fetch file row {file_id}"))?;

    let Some(file) = row else {
        log::warn!("file {file_id} was not found in db");
        return Ok(());
    };

    if file.status != STATUS_VALIDATING {
        log::debug!("skip file {}: status is {}", file.id, file.status);
        return Ok(());
    }

    let mut current_path = PathBuf::from(&file.path);
    let current_ext = current_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (detected_ext, matcher_type) = detect_file_kind(&current_path, &current_ext)?;
    let media_type = map_matcher_type(matcher_type);

    if !detected_ext.is_empty()
        && detected_ext != "unknown"
        && detected_ext != "unknow"
        && detected_ext != current_ext
    {
        let is_jpeg_case = (detected_ext == "jpg" && current_ext == "jpeg")
            || (detected_ext == "jpeg" && current_ext == "jpg");

        if !is_jpeg_case {
            let new_path = current_path.with_extension(&detected_ext);
            tokio::fs::rename(&current_path, &new_path)
                .await
                .with_context(|| {
                    format!(
                        "failed to rename {} -> {}",
                        current_path.display(),
                        new_path.display()
                    )
                })?;
            current_path = new_path;
        }
    }

    if media_type.is_none() {
        log::warn!("unsupported file {} ({})", file.id, current_path.display());
        let _ = tokio::fs::remove_file(&current_path).await;
        mark_failed(pool, file.id).await?;
        return Ok(());
    }

    let media_type_value = media_type.expect("checked above");
    sqlx::query(
        r#"
        UPDATE files
        SET path = $2,
            media_type = $3,
            meta = jsonb_build_object('extension', $4),
            status = $5
        WHERE id = $1
        "#,
    )
    .bind(file.id)
    .bind(current_path.to_string_lossy().to_string())
    .bind(media_type_value)
    .bind(&detected_ext)
    .bind(STATUS_THUMBNAILING)
    .execute(pool)
    .await
    .with_context(|| format!("failed to update file {}", file.id))?;

    redis_conn
        .rpush::<_, _, ()>(thumbnailer_queue, file.id.to_string())
        .await
        .with_context(|| format!("failed to enqueue thumbnail job for {}", file.id))?;

    log::info!(
        "file {} validated (type={media_type_value}, ext={}) and sent to thumbnailer",
        file.id,
        detected_ext
    );

    Ok(())
}

fn map_matcher_type(matcher_type: MatcherType) -> Option<i16> {
    match matcher_type {
        MatcherType::Image => Some(0),
        MatcherType::Video => Some(1),
        MatcherType::Audio => Some(2),
        _ => None,
    }
}

fn detect_file_kind(path: &Path, current_ext: &str) -> anyhow::Result<(String, MatcherType)> {
    let kind_opt = infer::get_from_path(path)?;

    let (mut detected_extension, matcher_type) = match kind_opt {
        Some(kind) => (kind.extension().to_string(), kind.matcher_type().to_owned()),
        None => {
            let matcher = match current_ext {
                "mp4" | "webm" | "mkv" | "mov" | "wmv" => MatcherType::Video,
                "mp3" | "wav" | "aac" | "flac" | "m4a" => MatcherType::Audio,
                _ => MatcherType::Image,
            };
            (current_ext.to_string(), matcher)
        }
    };

    if detected_extension == "webm" || detected_extension == "mkv" {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = [0u8; 512];
        let n = std::io::Read::read(&mut file, &mut buffer)?;
        let header = &buffer[..n];

        if header.windows(8).any(|window| window == b"matroska") {
            detected_extension = "mkv".to_string();
        } else {
            detected_extension = "webm".to_string();
        }
    }

    Ok((detected_extension, matcher_type))
}

async fn mark_failed(pool: &sqlx::PgPool, file_id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE files SET status = $2 WHERE id = $1")
        .bind(file_id)
        .bind(STATUS_FAILED)
        .execute(pool)
        .await
        .with_context(|| format!("failed to mark file {} failed", file_id))?;
    Ok(())
}
