import type { CollectionsRepository } from "$lib/application/ports/collections-repository";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";

export const searchCollections = (repo: CollectionsRepository, query: SearchPostsQuery) => {
    return repo.searchCollections(query);
};
