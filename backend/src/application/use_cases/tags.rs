// Tag Use-Case

use crate::application::contracts::NewTag;
use crate::application::ports::TagRepository;
use crate::domain::model::{RepoError, Tag};

pub struct CreateTagUseCase<R: TagRepository> {
    pub repo: R,
}

impl<R: TagRepository> CreateTagUseCase<R> {
    pub async fn execute(&self, tags: Vec<NewTag>) -> Result<Vec<Tag>, RepoError> {
        self.repo.get_or_create(tags).await
    }
}

pub struct SearchTagsUseCase<TR> {
    pub repo: TR,
}

impl<TR: TagRepository> SearchTagsUseCase<TR> {
    pub async fn execute(&self, query: &str, limit: i64) -> Result<Vec<Tag>, RepoError> {
        self.repo.search(query, limit).await
    }
}
