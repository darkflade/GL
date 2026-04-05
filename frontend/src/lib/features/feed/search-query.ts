import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type { Tag } from "$lib/domain/models/tag";
import { deserializeQuery, parseSearch, serializeQuery } from "$lib/utils/search";

export type PaginationMode = SearchPostsQuery["cursor"]["mode"];
export const DEFAULT_KEYSET_LIMIT = 20;

export const EMPTY_SEARCH_QUERY: SearchPostsQuery = {
    tag_query: {
        must: [],
        should: [],
        must_not: [],
    },
    text_query: "",
    cursor: {
        mode: "keyset",
        limit: DEFAULT_KEYSET_LIMIT,
    },
};

export function cursorForMode(mode: PaginationMode): SearchPostsQuery["cursor"] {
    if (mode === "keyset") {
        return {
            mode: "keyset",
            limit: DEFAULT_KEYSET_LIMIT,
        };
    }

    return {
        mode: "offset",
        page: 0,
    };
}

export function applyPaginationMode(query: SearchPostsQuery, mode: PaginationMode): SearchPostsQuery {
    if (query.cursor.mode === mode) {
        return query;
    }

    return {
        ...query,
        cursor: cursorForMode(mode),
    };
}

export function queryFromTags(tags: Tag[], mode: PaginationMode): SearchPostsQuery {
    return {
        tag_query: {
            must: tags.map((tag) => tag.name),
            should: [],
            must_not: [],
        },
        text_query: "",
        cursor: cursorForMode(mode),
    };
}

export function queryFromText(searchText: string, mode: PaginationMode): SearchPostsQuery {
    return applyPaginationMode(parseSearch(searchText), mode);
}

export function queryFromUrl(params: URLSearchParams): SearchPostsQuery {
    return deserializeQuery(params);
}

export function buildSearchHref(pathname: string, query: SearchPostsQuery): string {
    const queryString = serializeQuery(query);
    return queryString ? `${pathname}?${queryString}` : pathname;
}
