use crate::domain::model::FileStatus;
use crate::storage::file_storage::media_library::{ImportFile, MediaFile};
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
pub struct FilePath {
    pub id: Uuid,
    pub path: String,
}

#[derive(sqlx::FromRow)]
pub struct RepairRecord {
    pub path: String,
    pub hash: Option<String>,
    pub media_type: i16,
    pub status: i16,
}

pub async fn record(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<RepairRecord>> {
    sqlx::query_as("SELECT path, hash, media_type, status FROM files WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn paths_after(pool: &PgPool, after: Uuid) -> sqlx::Result<Vec<FilePath>> {
    sqlx::query_as("SELECT id, path FROM files WHERE id > $1 ORDER BY id LIMIT 500")
        .bind(after)
        .fetch_all(pool)
        .await
}

async fn lock_file_changes(tx: &mut Transaction<'_, Postgres>) -> sqlx::Result<()> {
    // Serialize import and repair checks without requiring existing libraries to deduplicate rows.
    sqlx::query("SELECT pg_advisory_xact_lock(734021)")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn import(pool: &PgPool, file: ImportFile) -> anyhow::Result<bool> {
    let mut tx = pool.begin().await?;
    lock_file_changes(&mut tx).await?;
    let existing: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM files WHERE path = $1 OR hash = $2)")
            .bind(&file.media.path)
            .bind(&file.media.hash)
            .fetch_one(&mut *tx)
            .await?;
    if existing {
        return Ok(false);
    }

    let file_id = Uuid::now_v7();
    let post_id = Uuid::now_v7();
    sqlx::query("INSERT INTO files (id, path, hash, media_type, meta, status) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(file_id)
        .bind(file.media.path)
        .bind(file.media.hash)
        .bind(i16::from(file.media.media_type))
        .bind(serde_json::to_value(file.media.meta)?)
        .bind(i16::from(FileStatus::Ready))
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO posts (id, title, file_id) VALUES ($1, $2, $3)")
        .bind(post_id)
        .bind(file.title)
        .bind(file_id)
        .execute(&mut *tx)
        .await?;

    for note in file.notes {
        sqlx::query(
            "INSERT INTO post_notes (id, post_id, text, pos_x, pos_y) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::now_v7())
        .bind(post_id)
        .bind(note.text)
        .bind(note.pos_x)
        .bind(note.pos_y)
        .execute(&mut *tx)
        .await?;
    }
    for (category, name) in file.tags {
        let tag_id: Uuid = sqlx::query_scalar(
            "INSERT INTO tags (id, category, name) VALUES ($1, $2, $3)
             ON CONFLICT (name, category) DO UPDATE SET name = EXCLUDED.name RETURNING id",
        )
        .bind(Uuid::now_v7())
        .bind(category)
        .bind(name)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)")
            .bind(post_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(true)
}

pub async fn repair(
    pool: &PgPool,
    id: Uuid,
    previous: &RepairRecord,
    file: MediaFile,
) -> anyhow::Result<bool> {
    let mut tx = pool.begin().await?;
    lock_file_changes(&mut tx).await?;
    let result = sqlx::query(
        "UPDATE files SET path = $2, hash = $3, status = $4,
           meta = CASE WHEN hash IS NULL THEN $5 ELSE COALESCE(meta, '{}'::jsonb) || jsonb_build_object('extension', $6::text) END
         WHERE id = $1 AND path = $7 AND status = $8 AND hash IS NOT DISTINCT FROM $9
           AND NOT EXISTS (SELECT 1 FROM files other WHERE other.path = $2 AND other.id <> $1)",
    )
    .bind(id)
    .bind(file.path)
    .bind(file.hash)
    .bind(i16::from(FileStatus::Ready))
    .bind(serde_json::to_value(&file.meta)?)
    .bind(file.meta.extension)
    .bind(&previous.path)
    .bind(previous.status)
    .bind(&previous.hash)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(result.rows_affected() == 1)
}
