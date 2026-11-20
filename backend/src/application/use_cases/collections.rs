use crate::application::contracts::{
    KeysetCursor, NewCollection, CollectionQuery, SearchCollectionsResponse, UpdateCollection,
};
use crate::application::ports::CollectionRepository;
use crate::domain::model::{Collection, CollectionID, RepoError, UserID};

// Collection Use-Case

pub struct GetCollectionUseCase<PLR> {
    pub repo: PLR,
}

impl<PLR: CollectionRepository> GetCollectionUseCase<PLR> {
    pub async fn execute(
        &self,
        user_id: UserID,
        collection_id: CollectionID,
    ) -> Result<Collection, RepoError> {
        self.repo.get(user_id, collection_id).await
    }
}

pub struct SearchCollectionsUseCase<PLR> {
    pub repo: PLR,
}

pub struct CreateCollectionUseCase<PLR> {
    pub repo: PLR,
}

pub struct DeleteCollectionUseCase<PLR> {
    pub repo: PLR,
}

pub struct UpdateCollectionUseCase<PLR> {
    pub repo: PLR,
}

impl<PLR: CollectionRepository> SearchCollectionsUseCase<PLR> {
    pub async fn execute(
        &self,
        user_id: UserID,
        query: CollectionQuery,
        cursor: KeysetCursor,
    ) -> Result<SearchCollectionsResponse, RepoError> {
        self.repo.search(user_id, query, cursor).await
    }
}

pub struct GetAllCollectionsUseCase<PLR> {
    pub repo: PLR,
}

impl<PLR: CollectionRepository> GetAllCollectionsUseCase<PLR> {
    pub async fn execute(
        &self,
        user_id: UserID,
        cursor: KeysetCursor,
    ) -> Result<SearchCollectionsResponse, RepoError> {
        self.repo.get_all(user_id, cursor).await
    }
}

impl<PLR: CollectionRepository> DeleteCollectionUseCase<PLR> {
    pub async fn execute(&self, user_id: UserID, collection_id: CollectionID) -> Result<(), RepoError> {
        self.repo.delete(user_id, collection_id).await
    }
}

impl<PLR: CollectionRepository> CreateCollectionUseCase<PLR> {
    pub async fn execute(
        &self,
        user_id: UserID,
        new_collection: NewCollection,
    ) -> Result<CollectionID, RepoError> {
        self.repo.create(user_id, new_collection).await
    }
}

impl<PLR: CollectionRepository> UpdateCollectionUseCase<PLR> {
    pub async fn execute(
        &self,
        user_id: UserID,
        collection_id: CollectionID,
        update_collection: UpdateCollection,
    ) -> Result<(), RepoError> {
        self.repo
            .update(user_id, collection_id, update_collection)
            .await
    }
}
