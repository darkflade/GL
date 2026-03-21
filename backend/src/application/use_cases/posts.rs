use crate::application::contracts::{
    Cursor, KeysetCursor, NewPost, NewTag, SearchPostsKeysetResponse, SearchPostsOffsetResponse,
    TagQuery, UpdatePost,
};
use crate::application::ports::{FileRepository, PostRepository, TagRepository};
use crate::domain::files::FileStorage;
use crate::domain::model::{ByteStream, File, FileStatus, FileType, Post, PostID, RepoError};
use actix_web::mime::Mime;
use redis::AsyncCommands;
use std::path::PathBuf;
use uuid::Uuid;

// Post Use-Case
pub struct CreatePostUseCase<PR, TR, FR, FS> {
    pub posts: PR,
    pub tags: TR,
    pub files: FR,
    pub storage: FS,
}

impl<PR: PostRepository, TR: TagRepository, FR: FileRepository, FS: FileStorage>
    CreatePostUseCase<PR, TR, FR, FS>
{
    async fn enqueue_file_for_validation(file_id: Uuid) -> Result<(), RepoError> {
        let redis_url = std::env::var("REDIS_URL").map_err(|err| {
            log::error!("posts.create missing REDIS_URL: {err}");
            RepoError::StorageError
        })?;
        let queue_name = std::env::var("FILE_VALIDATOR_QUEUE")
            .unwrap_or_else(|_| "file_validator:jobs".to_string());

        let client = redis::Client::open(redis_url).map_err(|err| {
            log::error!("posts.create failed to create redis client: {err}");
            RepoError::StorageError
        })?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                log::error!("posts.create failed to connect redis: {err}");
                RepoError::StorageError
            })?;

        conn.rpush::<_, _, ()>(queue_name, file_id.to_string())
            .await
            .map_err(|err| {
                log::error!("posts.create failed to enqueue file {file_id}: {err}");
                RepoError::StorageError
            })?;

        Ok(())
    }

    pub async fn execute(
        &self,
        title: String,
        stream: ByteStream,
        file_ext: Option<&str>,
        _mime_type: Option<Mime>,
        tags: Vec<NewTag>,
    ) -> Result<PostID, RepoError> {
        let (file_id, stored_path) = self
            .storage
            .save_stream(stream, file_ext)
            .await
            .map_err(|_| RepoError::StorageError)?;

        let file_model = File {
            id: file_id,
            path: PathBuf::from(&stored_path),
            media_type: FileType::Picture,
            status: FileStatus::Validating,
            hash: None,
            meta: None,
            created_at: None,
            thumbnail: None,
        };

        self.files.create(file_model).await?;
        self.files
            .set_status(file_id, FileStatus::Validating)
            .await?;

        let created_tags = self.tags.get_or_create(tags).await?;
        let tag_ids: Vec<Uuid> = created_tags.into_iter().map(|t| t.id).collect();

        let new_post = NewPost {
            id: Uuid::now_v7(),
            title,
            file_id,
            tag_ids,
        };

        let post_id = self.posts.create(new_post).await?;
        if let Err(err) = Self::enqueue_file_for_validation(file_id).await {
            let _ = self.files.mark_failed(file_id).await;
            return Err(err);
        }

        Ok(post_id)
    }
}

pub struct SearchPostsUseCase<PR> {
    pub repo: PR,
}

pub struct GetPostUseCase<PR> {
    pub repo: PR,
}

pub struct DeletePostUseCase<PR> {
    pub repo: PR,
}

pub struct UpdatePostUseCase<PR> {
    pub repo: PR,
}

pub struct GetAllPostsUseCase<PR> {
    pub repo: PR,
}

pub struct SearchPostsKeysetUseCase<PR> {
    pub repo: PR,
}

pub struct GetAllPostsKeysetUseCase<PR> {
    pub repo: PR,
}

impl<PR: PostRepository> SearchPostsUseCase<PR> {
    pub async fn execute(
        &self,
        query: TagQuery,
        cursor: Cursor,
    ) -> Result<SearchPostsOffsetResponse, RepoError> {
        self.repo.search(query, cursor).await
    }
}
impl<PR: PostRepository> GetAllPostsUseCase<PR> {
    pub async fn execute(&self, cursor: Cursor) -> Result<SearchPostsOffsetResponse, RepoError> {
        self.repo.get_all(cursor).await
    }
}

impl<PR: PostRepository> SearchPostsKeysetUseCase<PR> {
    pub async fn execute(
        &self,
        query: TagQuery,
        cursor: KeysetCursor,
    ) -> Result<SearchPostsKeysetResponse, RepoError> {
        self.repo.search_keyset(query, cursor).await
    }
}

impl<PR: PostRepository> GetAllPostsKeysetUseCase<PR> {
    pub async fn execute(
        &self,
        cursor: KeysetCursor,
    ) -> Result<SearchPostsKeysetResponse, RepoError> {
        self.repo.get_all_keyset(cursor).await
    }
}

impl<PR: PostRepository> GetPostUseCase<PR> {
    pub async fn execute(&self, id: PostID) -> Result<Post, RepoError> {
        self.repo.get(id).await
    }
}

impl<PR: PostRepository> DeletePostUseCase<PR> {
    pub async fn execute(&self, id: PostID) -> Result<(), RepoError> {
        self.repo.delete(id).await
    }
}

impl<PR: PostRepository> UpdatePostUseCase<PR> {
    pub async fn execute(&self, id: PostID, update_post: UpdatePost) -> Result<(), RepoError> {
        self.repo.update(id, update_post).await
    }
}
