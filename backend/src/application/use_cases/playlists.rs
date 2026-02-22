use uuid::Uuid;
use crate::domain::model::{Cursor, Playlist, PlaylistID, PlaylistQuery, PlaylistSummary, RepoError, SearchPlaylistsResponse, TagQuery, UserID};
use crate::domain::repository::PlaylistRepository;

// Playlist Use-Case
pub struct SearchPlaylistsUseCase<R: PlaylistRepository> {
    pub repo: R,
}

pub struct GetPlaylistUseCase<R: PlaylistRepository> {
    pub repo: R,
}
impl<R: PlaylistRepository> SearchPlaylistsUseCase<R> {

    pub async fn execute(&self, user_id: UserID, query: TagQuery, cursor: Cursor) -> Result<SearchPlaylistsResponse, RepoError> {
        self.repo.search_by_tags(user_id, query, cursor).await
    }
}
impl<R: PlaylistRepository> GetPlaylistUseCase<R> {
    pub async fn execute(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<Playlist, RepoError> {
        self.repo.get(user_id, playlist_id).await
    }
}