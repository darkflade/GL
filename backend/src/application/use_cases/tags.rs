// Tag Use-Case

use crate::application::contracts::{
    KeysetCursor, NewTag, SearchTagRelationsResponse, SearchTagsResponse, TagBatchUpdate,
    TagRelationsBatchUpdate,
};
use crate::application::ports::TagRepository;
use crate::domain::model::{RepoError, Tag, TagID};

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

pub struct ListTagsKeysetUseCase<TR> {
    pub repo: TR,
}

impl<TR: TagRepository> ListTagsKeysetUseCase<TR> {
    pub async fn execute(&self, cursor: KeysetCursor) -> Result<SearchTagsResponse, RepoError> {
        self.repo.list_keyset(cursor).await
    }
}

pub struct GetRelatedTagsUseCase<TR> {
    pub repo: TR,
}

impl<TR: TagRepository> GetRelatedTagsUseCase<TR> {
    pub async fn execute(&self, tag_id: TagID) -> Result<Vec<Tag>, RepoError> {
        self.repo.get_related(tag_id).await
    }
}

pub struct ListTagRelationsKeysetUseCase<TR> {
    pub repo: TR,
}

impl<TR: TagRepository> ListTagRelationsKeysetUseCase<TR> {
    pub async fn execute(
        &self,
        cursor: KeysetCursor,
    ) -> Result<SearchTagRelationsResponse, RepoError> {
        self.repo.list_relations_keyset(cursor).await
    }
}

pub struct UpdateTagsUseCase<TR> {
    pub repo: TR,
}

impl<TR: TagRepository> UpdateTagsUseCase<TR> {
    pub async fn execute(&self, update: TagBatchUpdate) -> Result<(), RepoError> {
        self.repo.update_tags(update).await
    }
}

pub struct UpdateTagRelationsUseCase<TR> {
    pub repo: TR,
}

impl<TR: TagRepository> UpdateTagRelationsUseCase<TR> {
    pub async fn execute(&self, update: TagRelationsBatchUpdate) -> Result<(), RepoError> {
        self.repo.update_relations(update).await
    }
}
