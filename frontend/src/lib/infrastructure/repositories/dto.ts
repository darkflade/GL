import type { Post } from "$lib/domain";

export interface NextKeysetCursor {
    mode: "keyset";
    last_id: string;
    last_score: number;
    limit: number;
}

export interface SearchPostsResponseOffset {
    posts: Post[];
    total_pages: number;
    total_count?: number;
}

export interface SearchPostsResponseKeyset {
    posts: Post[];
    next_cursor?: NextKeysetCursor;
    has_next: boolean;
}

export type SearchPostsResponse = SearchPostsResponseOffset | SearchPostsResponseKeyset;
