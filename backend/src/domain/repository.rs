use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::model::{Cursor, File, FileID, KeysetCursor, NewPost, NewTag, NewUser, Playlist, PlaylistID, PlaylistQuery, Post, PostID, RepoError, SearchPlaylistsResponse, SearchPostsKeysetResponse, SearchPostsOffsetResponse, Tag, TagID, TagQuery, User, UserID};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: NewPost, tag_ids: &[TagID]) -> Result<PostID, RepoError> ;
    async fn get(&self, id: PostID) -> Result<Post, RepoError>;
    //Offset search
    async fn search(&self, query: TagQuery, cursor: Cursor) -> Result<SearchPostsOffsetResponse, RepoError>;
    async fn get_all(&self, cursor: Cursor) -> Result<SearchPostsOffsetResponse, RepoError>;
    //Keyset search
    async fn search_keyset(&self, query: TagQuery, cursor: KeysetCursor) -> Result<SearchPostsKeysetResponse, RepoError>;
    async fn get_all_keyset(&self, cursor: KeysetCursor) -> Result<SearchPostsKeysetResponse, RepoError>;
}

#[async_trait]
pub trait PlaylistRepository: Send + Sync {
    async fn get(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<Playlist, RepoError>;

    //Only keyset search in playlists
    async fn search(&self, user_id: UserID, query: PlaylistQuery, cursor: KeysetCursor) -> Result<SearchPlaylistsResponse, RepoError>;
    
    async fn get_all(&self, user_id: UserID, cursor: KeysetCursor) -> Result<SearchPlaylistsResponse, RepoError>;
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
