use crate::application::contracts::{KeysetCursor, PlaylistQuery, SearchPlaylistsResponse};
use crate::application::ports::PlaylistRepository;
use crate::domain::model::{Playlist, PlaylistID, RepoError, UserID};

// Playlist Use-Case

pub struct GetPlaylistUseCase<PLR> {
    pub repo: PLR,
}

impl<PLR: PlaylistRepository> GetPlaylistUseCase<PLR> {
    pub async fn execute(
        &self,
        user_id: UserID,
        playlist_id: PlaylistID,
    ) -> Result<Playlist, RepoError> {
        self.repo.get(user_id, playlist_id).await
    }
}

pub struct SearchPlaylistsUseCase<PLR> {
    pub repo: PLR,
}

pub struct DeletePlaylistUseCase<PLR> {
    pub repo: PLR,
}

impl<PLR: PlaylistRepository> SearchPlaylistsUseCase<PLR> {
    pub async fn execute(
        &self,
        user_id: UserID,
        query: PlaylistQuery,
        cursor: KeysetCursor,
    ) -> Result<SearchPlaylistsResponse, RepoError> {
        self.repo.search(user_id, query, cursor).await
    }
}

pub struct GetAllPlaylistsUseCase<PLR> {
    pub repo: PLR,
}

impl<PLR: PlaylistRepository> GetAllPlaylistsUseCase<PLR> {
    pub async fn execute(
        &self,
        user_id: UserID,
        cursor: KeysetCursor,
    ) -> Result<SearchPlaylistsResponse, RepoError> {
        self.repo.get_all(user_id, cursor).await
    }
}

impl<PLR: PlaylistRepository> DeletePlaylistUseCase<PLR> {
    pub async fn execute(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<(), RepoError> {
        self.repo.delete(user_id, playlist_id).await
    }
}
