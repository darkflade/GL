// Tag Use-Case

use crate::domain::model::{NewTag, RepoError, Tag};
use crate::domain::repository::TagRepository;

pub struct CreateTagUseCase<R: TagRepository> {
    pub repo: R,
}

impl<R: TagRepository> CreateTagUseCase<R> {
    pub async fn execute(&self, tags: Vec<NewTag>) -> Result<Vec<Tag>, RepoError> {
        self.repo.get_or_create(tags).await
    }
}