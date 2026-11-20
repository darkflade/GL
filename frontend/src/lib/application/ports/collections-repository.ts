import type { Collection } from "$lib/domain/models/collection";
import type { NewCollection, SearchPostsQuery, UpdateCollection, UUID } from "$lib/domain";
import type { SearchCollectionsResponse } from "$lib/infrastructure/repositories/dto";

export interface CollectionsRepository {
    searchCollections(query: SearchPostsQuery): Promise<SearchCollectionsResponse>;
    deleteCollection(id: UUID): Promise<void>;
    getCollectionByID(id: UUID): Promise<Collection>;
    createCollection(payload: NewCollection): Promise<UUID>;
    updateCollection(id: UUID, payload: UpdateCollection): Promise<void>;
}
