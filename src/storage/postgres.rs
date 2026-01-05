use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::domain::model::{NewPost, NewTag, Playlist, PlaylistContent, PlaylistID, PlaylistItem, PlaylistQuery, PlaylistSummary, PlaylistWithItems, Post, PostID, RepoError, Tag, TagID, TagQuery};
use crate::domain::repository::{PlaylistRepository, PostRepository, TagRepository};

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(
        &self,
        post: NewPost,
        tag_ids: &[TagID],
    ) -> Result<PostID, RepoError> {
        let mut tx = self.pool.begin().await.map_err(|_| RepoError::StorageError)?;

        sqlx::query!(
            "INSERT INTO posts (id, title, file_id) VALUES ($1, $2, $3)",
            post.id,
            post.title,
            post.file_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| RepoError::StorageError)?;

        for tag_id in tag_ids {
            sqlx::query!(
            "INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)",
            post.id,
            tag_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| RepoError::StorageError)?;

        }

        tx.commit().await.map_err(|_| RepoError::StorageError)?;

        Ok(post.id)
    }

    async fn get(&self, id: PostID) -> Result<Post, RepoError> {
        let row = sqlx::query!(
            r#"
            SELECT p.id, p.title, p.file_id
            FROM posts p
            WHERE p.id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?;

        let row = row.ok_or(RepoError::NotFound)?;

        let tags = sqlx::query!(
            r#"
            SELECT tag_id
            FROM post_tags
            WHERE post_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepoError::StorageError)?;

        Ok(Post {
            id: row.id,
            title: row.title,
            file_id: row.file_id,
            tag_ids: tags.into_iter().map(|r| r.tag_id).collect(),
            notes: None,
        })
    }

    async fn search(&self, query: TagQuery) -> Result<Vec<Post>, RepoError> {
        let rows = sqlx::query!(
            r#"
            SELECT p.id, p.title, p.file_id
            FROM posts p
            JOIN post_tags pt ON pt.post_id = p.id
            WHERE
                ($1::uuid[] IS NULL OR pt.tag_id = ANY($1))
            GROUP BY p.id
            HAVING
                COUNT(DISTINCT CASE WHEN pt.tag_id = ANY($2) THEN pt.tag_id END) = cardinality($2)
            "#,
            &query.should[..],
            &query.must[..]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepoError::StorageError)?;

        Ok(rows.into_iter().map(|r| Post {
            id: r.id,
            title: r.title,
            file_id: r.file_id,
            tag_ids: vec![],
            notes: None,
        }).collect())
    }

}


#[derive(Clone)]
pub struct PostgresTagRepository {
    pool: PgPool,
}

impl PostgresTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for PostgresTagRepository {

    async fn get_or_create(&self, tags: Vec<NewTag>) -> Result<Vec<Tag>, RepoError> {

        let mut result = Vec::new();
        for new_tag in tags {
            let rec = sqlx::query!(
                "INSERT INTO tags (id, category, value) VALUES ($1, $2, $3)
                 ON CONFLICT (category, value) DO UPDATE SET value = EXCLUDED.value
                 RETURNING id, category, value",
                Uuid::new_v4(),
                new_tag.category as i32,
                new_tag.value
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?;

            result.push(Tag {
                id: rec.id,
                category: unsafe { std::mem::transmute(rec.category as i8) },
                value: rec.value
            });
        }
        Ok(result)
    }
    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Tag>, RepoError> {
        let pattern =  format!("{}%", query);
        let rows = sqlx::query!(
            "SELECT id, category, value FROM tags WHERE value ILIKE $1 LIMIT $2",
            pattern,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepoError::StorageError)?;

        Ok(rows.into_iter().map(|r| Tag {
            id: r.id,
            category: unsafe { std::mem::transmute(r.category as i8) },
            value: r.value,
        }).collect())
    }
}

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

        //TODO Make request for tags motherfucker
        Ok(
            Playlist {
                id: row.id,
                title: row.title,
                description: row.description.unwrap_or_default(),
                tag_ids: vec![],
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
                p.title as post_title,
                p.file_id as post_file_id
            FROM playlist_items pi
            LEFT JOIN posts p ON pi.post_id = p.id
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
                PlaylistContent::Post(Post {
                    id: p_id,
                    title: row.post_title,
                    tag_ids: vec![],
                    file_id: row.post_file_id,
                    notes: None,
                })
            } else {
                PlaylistContent::Note(row.note_text.unwrap_or_default())
            };

            PlaylistItem {
                id: row.item_id,
                playlist_id: id.clone(),
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

    async fn search(&self, query: PlaylistQuery) -> Result<Vec<Playlist>, RepoError> {
        Ok(vec![])
    }
}