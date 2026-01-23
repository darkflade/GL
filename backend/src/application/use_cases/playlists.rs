use crate::domain::model::{Playlist, PlaylistID, PlaylistQuery, RepoError};
use crate::domain::repository::PlaylistRepository;

// Playlist Use-Case
pub struct SearchPlaylistsUseCase<R: PlaylistRepository> {
    pub repo: R,
}

pub struct GetPlaylistUseCase<R: PlaylistRepository> {
    pub repo: R,
}
impl<R: PlaylistRepository> SearchPlaylistsUseCase<R> {

    pub async fn execute(&self, query: PlaylistQuery) -> Result<Vec<Playlist>, RepoError> {
        self.repo.search(query).await
    }
}
impl<R: PlaylistRepository> GetPlaylistUseCase<R> {
    pub async fn execute(&self, id: PlaylistID) -> Result<Playlist, RepoError> {
        self.repo.get(id).await
    }
}