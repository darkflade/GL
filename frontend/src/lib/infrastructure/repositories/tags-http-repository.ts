import { api } from "$lib/infrastructure/http/client";
import type { TagsRepository } from "$lib/application/ports/tags-repository";
import type { Tag } from "$lib/domain/models/tag";

export const tagsHttpRepository: TagsRepository = {
    searchTags: (query: string) => {
        return api.get<Tag[]>(`/tags/search?query=${query}`);
    },
};
