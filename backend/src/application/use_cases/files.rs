use crate::application::ports::FileRepository;
use crate::domain::model::{File, FileID, RepoError};

// File Use-Case
pub struct GetFileUseCase<FR> {
    pub repo: FR,
}

impl<FR: FileRepository> GetFileUseCase<FR> {
    pub async fn execute(&self, id: FileID) -> Result<File, RepoError> {
        self.repo.get(id).await
    }
}
