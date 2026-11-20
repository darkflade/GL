import type { CollectionsRepository } from "$lib/application/ports/collections-repository";
import type { NewCollection } from "$lib/domain";

export const createCollection = (repo: CollectionsRepository, payload: NewCollection) => {
    return repo.createCollection(payload);
};
