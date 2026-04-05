import type { Tag } from "$lib/domain/models/tag";
import type { SearchTagRelationsResponse, SearchTagsResponse } from "$lib/infrastructure/repositories/dto";
import type { KeysetCursor } from "$lib/domain/value-objects/search";

export interface TagsRepository {
    searchTags(query: string): Promise<Tag[]>;
    listTags(cursor?: KeysetCursor): Promise<SearchTagsResponse>;
    listTagRelations(cursor?: KeysetCursor): Promise<SearchTagRelationsResponse>;
}
