import { api } from "$lib/infrastructure/http/client";
import type { TagsRepository } from "$lib/application/ports/tags-repository";
import type { Tag } from "$lib/domain/models/tag";
import type { KeysetCursor } from "$lib/domain/value-objects/search";
import type { SearchTagRelationsResponse, SearchTagsResponse } from "$lib/infrastructure/repositories/dto";

function toQueryString(cursor?: KeysetCursor): string {
    const params = new URLSearchParams();
    params.set("mode", "keyset");

    if (cursor?.direction) params.set("direction", cursor.direction);
    if (cursor?.last_id) params.set("last_id", cursor.last_id);
    if (cursor?.last_score !== undefined) params.set("last_score", String(cursor.last_score));
    if (cursor?.limit !== undefined) params.set("limit", String(cursor.limit));

    const query = params.toString();
    return query ? `?${query}` : "";
}

export const tagsHttpRepository: TagsRepository = {
    searchTags: (query: string) => {
        return api.get<Tag[]>(`/tags/search?query=${query}`);
    },
    listTags: (cursor?: KeysetCursor) => {
        return api.get<SearchTagsResponse>(`/tags${toQueryString(cursor)}`);
    },
    listTagRelations: (cursor?: KeysetCursor) => {
        return api.get<SearchTagRelationsResponse>(`/tags/relations${toQueryString(cursor)}`);
    },
};
