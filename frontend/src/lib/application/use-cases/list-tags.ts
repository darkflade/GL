import type { TagsRepository } from "$lib/application/ports/tags-repository";
import type { KeysetCursor } from "$lib/domain/value-objects/search";

export const listTags = (repo: TagsRepository, cursor?: KeysetCursor) => {
    return repo.listTags(cursor);
};

export const listTagRelations = (repo: TagsRepository, cursor?: KeysetCursor) => {
    return repo.listTagRelations(cursor);
};
