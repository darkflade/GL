import { api } from "$lib/infrastructure/http/client";
import type { CollectionsRepository } from "$lib/application/ports/collections-repository";
import type { Collection } from "$lib/domain/models/collection";
import type { NewCollection, SearchPostsQuery, UpdateCollection, UUID } from "$lib/domain";
import type { SearchCollectionsResponse } from "$lib/infrastructure/repositories/dto";

export const collectionsHttpRepository: CollectionsRepository = {
    searchCollections: (query: SearchPostsQuery) => {
        const cursor =
            query.cursor.mode === "keyset"
                ? {
                    mode: "keyset" as const,
                    direction: query.cursor.direction,
                    last_id: query.cursor.last_id,
                    last_score: query.cursor.last_score,
                    limit: query.cursor.limit ?? 20,
                }
                : {
                    mode: "keyset" as const,
                    limit: query.cursor.page_size ?? 20,
                };

        return api.post<SearchCollectionsResponse>("/collections/search", {
            sort: query.sort,
            tag_query: {
                must: query.tag_query.must,
                should: query.tag_query.should,
                must_not: query.tag_query.must_not,
            },
            text_query: query.text_query,
            cursor,
        });
    },
    deleteCollection: (id: UUID) => api.delete<void>(`/collections/${id}`),
    getCollectionByID(id: UUID): Promise<Collection> {
        return api.get<Collection>(`/collections/${id}`);
    },
    createCollection(payload: NewCollection): Promise<UUID> {
        return api.post<UUID>("/collections", payload);
    },
    updateCollection(id: UUID, payload: UpdateCollection): Promise<void> {
        return api.patch<void>(`/collections/${id}`, payload);
    },
};
