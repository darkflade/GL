use std::path::PathBuf;
use crate::domain::model::{Cursor, File, KeysetCursor, PlaylistItem, PlaylistQuery, SearchPlaylistsResponse, Tag, TagQuery, UserID};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::model::{Playlist, PlaylistContent, PlaylistID, PlaylistSummary, Post, RepoError};
use crate::domain::repository::PlaylistRepository;
use crate::storage::postgres::dto::PlaylistItemResponse;

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

    async fn get(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<Playlist, RepoError> {
        //let playlist = self.get(user_id.clone(), playlist_id.clone()).await?;

        //TODO add user check
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
            playlist_id
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
                    thumbnail: None,
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
                position: row.position as u32,
                content,
                
            }
        }).collect();

        Ok(Playlist {
            id: playlist_id,
            title: "".to_string(),
            description: "".to_string(),
            tags: vec![],
            cover: None,
            items,
        })
    }

    // async fn get(&self, user_id: Uuid) -> Result<Vec<PlaylistSummary>, RepoError> {
    //     let rows = sqlx::query!(
    //         r#"
    //         SELECT
    //             p.id, p.title, p.description, p.cover_file_id,
    //             (SELECT COUNT(*)  FROM playlist_items pi  WHERE pi.playlist_id = p.id) as "item_count!",
    //             COALESCE(
    //                 json_agg(json_build_object('id', t.id, 'value', t.name, 'category', t.category))
    //                 FILTER (WHERE t.id IS NOT NULL),
    //                 '[]'
    //             ) as "tags!"
    //         FROM playlists p
    //         LEFT JOIN playlist_tags pt ON p.id = pt.playlist_id
    //         LEFT JOIN tags t ON pt.tag_id = t.id
    //         WHERE p.owner_id = $1
    //         GROUP BY p.id
    //         "#,
    //         user_id
    //     )
    //         .fetch_all(&self.pool)
    //         .await
    //         .map_err(|e| {
    //             println!("[DB] Error: {:?}", e);
    //             RepoError::StorageError
    //         })?;
    //
    //     let summaries = rows.into_iter().map(|r| {
    //         let tags: Vec<Tag> = serde_json::from_value(r.tags).unwrap_or_default();
    //
    //         PlaylistSummary {
    //             id: r.id,
    //             title: r.title,
    //             description: r.description.unwrap_or_default(),
    //             cover: r.cover_file_id,
    //             item_count: r.item_count,
    //             tags,
    //         }
    //     }).collect();
    //
    //     Ok(summaries)
    // }

    //TODO Make search
    async fn search(&self, user_id: UserID, query: PlaylistQuery, cursor: KeysetCursor) -> Result<SearchPlaylistsResponse, RepoError> {
        log::debug!("playlists.search_by_tags user={user_id} query={query:?} cursor={cursor:?}");
        let empty_resp = SearchPlaylistsResponse::default();
        Ok(empty_resp)
    }

    async fn get_all(&self, user_id: UserID, cursor: KeysetCursor) -> Result<SearchPlaylistsResponse, RepoError> {
        log::debug!("playlists.search_by_tags user={user_id} cursor={cursor:?}");
        let empty_resp = SearchPlaylistsResponse::default();
        Ok(empty_resp)
    }
}
