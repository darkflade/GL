import type { Post } from "$lib/domain";
import type { PlaylistSummary } from "$lib/domain/models/playlist";
import type { Tag, TagRelation } from "$lib/domain/models/tag";
import type { KeysetDirection } from "$lib/domain/value-objects/search";

export interface KeysetCursorDto {
    mode: "keyset";
    direction: KeysetDirection;
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
    next_cursor?: KeysetCursorDto;
    prev_cursor?: KeysetCursorDto;
    has_next: boolean;
    has_prev: boolean;
}

export type SearchPostsResponse = SearchPostsResponseOffset | SearchPostsResponseKeyset;

export interface SearchPlaylistsResponse {
    playlists: PlaylistSummary[];
    next_cursor?: KeysetCursorDto;
    prev_cursor?: KeysetCursorDto;
    has_next: boolean;
    has_prev: boolean;
}

export interface SearchTagsResponse {
    tags: Tag[];
    next_cursor?: KeysetCursorDto | null;
    prev_cursor?: KeysetCursorDto | null;
    has_next: boolean;
    has_prev: boolean;
}

export interface SearchTagRelationsResponse {
    relations: TagRelation[];
    next_cursor?: KeysetCursorDto | null;
    prev_cursor?: KeysetCursorDto | null;
    has_next: boolean;
    has_prev: boolean;
}
