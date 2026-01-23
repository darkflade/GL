use std::path::PathBuf;
use crate::domain::model::{File, PlaylistQuery, Tag};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::model::{Playlist, PlaylistContent, PlaylistID, PlaylistItem, PlaylistSummary, PlaylistWithItems, Post, RepoError};
use crate::domain::repository::PlaylistRepository;

#[derive(Clone)]
pub struct PostgresPlaylistRepository {
    pool: PgPool,
}

impl PostgresPlaylistRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlaylistRepository for PostgresPlaylistRepository {

    async fn get(&self, id: PlaylistID) -> Result<Playlist, RepoError> {
        let row = sqlx::query!(
            "SELECT id, title, description, cover_file_id FROM playlists WHERE id = $1",
            id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?
            .ok_or(RepoError::NotFound)?;

        Ok(
            Playlist {
                id: row.id,
                title: row.title,
                description: row.description.unwrap_or_default(),
                tags: vec![],
                cover: row.cover_file_id,
            }
        )
    }

    async fn get_with_items(&self, id: PlaylistID) -> Result<PlaylistWithItems, RepoError> {
        let playlist = self.get(id.clone()).await?;

        let items_rows = sqlx::query!(
            r#"
            SELECT
                pi.id as item_id,
                pi.position,
                pi.post_id,
                pi.note_text,

                -- Post Data
                p.title as post_title,
                p.description as post_desc,

                -- File Data
                f.id as file_id,
                f.path as file_path,
                f.hash as file_hash,
                f.media_type as "media_type: i16",
                f.meta as file_meta,
                f.created_at as created_at

            FROM playlist_items pi
            LEFT JOIN posts p ON pi.post_id = p.id
            LEFT JOIN files f ON p.file_id = f.id
            WHERE pi.playlist_id = $1
            ORDER BY pi.position ASC
            "#,
            id
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|r| RepoError::StorageError)?;

        let items = items_rows.into_iter().map(|row| {
            let content = if let Some(p_id) = row.post_id {
                let file_obj = File {
                    id: row.file_id,
                    path: PathBuf::from(row.file_path),
                    hash: row.file_hash,
                    media_type: row.media_type.into(),
                    meta: serde_json::from_value(row.file_meta.unwrap_or_default()).unwrap_or_default(),
                    created_at: row.created_at,
                };
                PlaylistContent::Post(Post {
                    id: p_id,
                    title: row.post_title,
                    description: row.post_desc,
                    //TODO Decide Tags
                    tags: vec![],
                    file: file_obj,
                    //TODO Playlist Notes
                    notes: vec![],
                })
            } else {
                PlaylistContent::Note(row.note_text.unwrap_or_default())
            };

            PlaylistItem {
                id: row.item_id,
                playlist_id: id,
                position: row.position as u32,
                content,
            }
        }).collect();

        Ok(PlaylistWithItems { playlist, items })
    }

    async fn get_by_user(&self, user_id: Uuid) -> Result<Vec<PlaylistSummary>, RepoError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                p.id, p.title, p.description, p.cover_file_id,
                (SELECT COUNT(*)  FROM playlist_items pi  WHERE pi.playlist_id = p.id) as "item_count!",
                COALESCE(
                    json_agg(json_build_object('id', t.id, 'value', t.value, 'category', t.category))
                    FILTER (WHERE t.id IS NOT NULL),
                    '[]'
                ) as "tags!"
            FROM playlists p
            LEFT JOIN playlist_tags pt ON p.id = pt.playlist_id
            LEFT JOIN tags t ON pt.tag_id = t.id
            WHERE p.owner_id = $1
            GROUP BY p.id
            "#,
            user_id
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                println!("[DB] Error: {:?}", e);
                RepoError::StorageError
            })?;

        let summaries = rows.into_iter().map(|r| {
            let tags: Vec<Tag> = serde_json::from_value(r.tags).unwrap_or_default();

            PlaylistSummary {
                id: r.id,
                title: r.title,
                description: r.description.unwrap_or_default(),
                cover: r.cover_file_id,
                item_count: r.item_count,
                tags,
            }
        }).collect();

        Ok(summaries)
    }

    //TODO Make search
    async fn search(&self, query: PlaylistQuery) -> Result<Vec<Playlist>, RepoError> {
        Ok(vec![])
    }
}