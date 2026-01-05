use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::model::{File, FileID, NewPost, NewTag, Playlist, PlaylistID, PlaylistQuery, PlaylistSummary, PlaylistWithItems, Post, PostID, RepoError, Tag, TagID, TagQuery};

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn get(&self, id: FileID) -> Result<File, RepoError>;
}

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: NewPost, tag_ids: &[TagID]) -> Result<PostID, RepoError> ;
    async fn get(&self, id: PostID) -> Result<Post, RepoError>;
    async fn search(&self, query: TagQuery) -> Result<Vec<Post>, RepoError>;
}

#[async_trait]
pub trait PlaylistRepository: Send + Sync {
    async fn get(&self, id: PlaylistID) -> Result<Playlist, RepoError>;

    async fn get_with_items(&self, id: PlaylistID) -> Result<PlaylistWithItems, RepoError>;

    async fn get_by_user(&self, user_id: Uuid) -> Result<Vec<PlaylistSummary>, RepoError>;
    async fn search(&self, query: PlaylistQuery) -> Result<Vec<Playlist>, RepoError>;
}


#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn get_or_create(&self, tag: Vec<NewTag>) -> Result<Vec<Tag>, RepoError>;

    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Tag>, RepoError>;
}

