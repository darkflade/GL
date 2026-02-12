import type { TagsRepository } from "$lib/application/ports/tags-repository";

export const searchTags = (repo: TagsRepository, query: string) => {
    return repo.searchTags(query);
};
