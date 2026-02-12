import type { Tag } from "$lib/domain/models/tag";

export interface TagsRepository {
    searchTags(query: string): Promise<Tag[]>;
}
