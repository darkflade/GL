import type { CollectionsRepository } from "$lib/application/ports/collections-repository";
import type { UpdateCollection, UUID } from "$lib/domain";

export const updateCollection = (repo: CollectionsRepository, id: UUID, payload: UpdateCollection) => {
    return repo.updateCollection(id, payload);
};
