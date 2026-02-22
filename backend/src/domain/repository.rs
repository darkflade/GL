use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::model::{Cursor, File, FileID, NewPost, NewTag, NewUser, Playlist, PlaylistID, PlaylistQuery, PlaylistSummary, Post, PostID, RepoError, SearchPlaylistsResponse, SearchPostsResponse, Tag, TagID, TagQuery, User, UserID};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: NewPost, tag_ids: &[TagID]) -> Result<PostID, RepoError> ;
    async fn get(&self, id: PostID) -> Result<Post, RepoError>;
    async fn search(&self, query: TagQuery, cursor: Cursor) -> Result<SearchPostsResponse, RepoError>;
    async fn get_all(&self, cursor: Cursor) -> Result<SearchPostsResponse, RepoError>;
}

#[async_trait]
pub trait PlaylistRepository: Send + Sync {
    async fn get(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<Playlist, RepoError>;

    //TODO clear

    // async fn get_with_items(&self, id: PlaylistID) -> Result<PlaylistWithItems, RepoError>;
    // async fn search_by_user(&self, user_id: Uuid, cursor: Uuid) -> Result<Vec<PlaylistSummary>, RepoError>;
    async fn search_by_tags(&self, user_id: UserID, query: TagQuery, cursor: Cursor) -> Result<SearchPlaylistsResponse, RepoError>;
    async fn get_all(&self, user_id: UserID, cursor: Cursor) -> Result<SearchPlaylistsResponse, RepoError>;
}


#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn get_or_create(&self, tag: Vec<NewTag>) -> Result<Vec<Tag>, RepoError>;

    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Tag>, RepoError>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {

    async fn find_by_id(&self, id: Uuid) -> Result<User, RepoError>;
    async fn find_by_username(&self, username: &str) -> Result<User, RepoError>;
    async fn create(&self, user: NewUser) -> Result<Uuid, RepoError>;
}

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn create(&self, file_info: File) -> Result<FileID,RepoError>;
    async fn get(&self, id: FileID) -> Result<File, RepoError>;
}

