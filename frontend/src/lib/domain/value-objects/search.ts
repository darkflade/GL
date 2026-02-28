export interface SearchPostsQuery {
    tag_query: TagQuery;
    text_query: string;
    cursor: Cursor;
}

export interface TagQuery {
    must: string[];
    should: string[];
    must_not: string[];
}

export type Cursor = OffsetCursor | KeysetCursor;

export interface OffsetCursor {
    mode: "offset";
    page: number;
    page_size?: number;
}

export interface KeysetCursor {
    mode: "keyset";
    last_id?: string;
    last_score?: number;
    limit?: number;
}
