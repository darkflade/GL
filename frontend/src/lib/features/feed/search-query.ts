import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type { Tag } from "$lib/domain/models/tag";
import { deserializeQuery, parseSearch, serializeQuery } from "$lib/utils/search";

export const EMPTY_SEARCH_QUERY: SearchPostsQuery = {
    tag_query: {
        must: [],
        should: [],
        must_not: [],
    },
    cursor: {
        page: 0,
    },
};

export function queryFromTags(tags: Tag[]): SearchPostsQuery {
    return {
        tag_query: {
            must: tags.map((tag) => tag.name),
            should: [],
            must_not: [],
        },
        cursor: {
            page: 0,
        }
    };
}

export function queryFromText(searchText: string): SearchPostsQuery {
    return parseSearch(searchText);
}

export function queryFromUrl(params: URLSearchParams): SearchPostsQuery {
    return deserializeQuery(params);
}

export function buildSearchHref(pathname: string, query: SearchPostsQuery): string {
    const queryString = serializeQuery(query);
    return queryString ? `${pathname}?${queryString}` : pathname;
}
