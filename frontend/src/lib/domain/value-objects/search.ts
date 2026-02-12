import type {UUID} from "$lib/domain";

export interface SearchPostsQuery {
    must: UUID[];
    should: UUID[];
    must_not: UUID[];
}

export interface SearchPlaylistQuery {
    tags: SearchPostsQuery | null;
    name: string | null;
}
