import type {UUID} from "$lib/domain";

export interface SearchPostsQuery {
    tag_query: TagQuery;
    cursor: Cursor;
}

export interface TagQuery {
    must: UUID[];
    should: UUID[];
    must_not: UUID[];
}

export interface Cursor {
    page: number;
}

export interface SearchPlaylistQuery {
    tags: SearchPostsQuery | null;
    name: string | null;
}
