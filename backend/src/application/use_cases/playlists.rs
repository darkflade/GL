use crate::domain::model::{KeysetCursor, Playlist, PlaylistID, PlaylistQuery, RepoError, SearchPlaylistsResponse, UserID};
use crate::domain::repository::PlaylistRepository;

// Playlist Use-Case

pub struct GetPlaylistUseCase<PLR> {
    pub repo: PLR,
}


impl<PLR: PlaylistRepository> GetPlaylistUseCase<PLR> {
    pub async fn execute(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<Playlist, RepoError> {
        self.repo.get(user_id, playlist_id).await
    }
}

pub struct SearchPlaylistsUseCase<PLR> {
    pub repo: PLR,
}

impl<PLR: PlaylistRepository> SearchPlaylistsUseCase<PLR> {

    pub async fn execute(&self, user_id: UserID, query: PlaylistQuery, cursor: KeysetCursor) -> Result<SearchPlaylistsResponse, RepoError> {
        self.repo.search(user_id, query, cursor).await
    }
}

pub struct GetAllPlaylistsUseCase<PLR> {
    pub repo: PLR,
}

impl<PLR: PlaylistRepository> GetAllPlaylistsUseCase<PLR> {
    
    pub async fn execute(&self, user_id: UserID, cursor: KeysetCursor) -> Result<SearchPlaylistsResponse, RepoError> {
        self.repo.get_all(user_id, cursor).await
    }
}
