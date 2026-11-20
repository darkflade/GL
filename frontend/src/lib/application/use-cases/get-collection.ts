import type { CollectionsRepository } from "$lib/application/ports/collections-repository";
import type { UUID } from "$lib/domain";

export const getCollection = (repo: CollectionsRepository, id: UUID) => {
    return repo.getCollectionByID(id);
};
