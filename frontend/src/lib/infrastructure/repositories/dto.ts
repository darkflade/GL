import type {Post} from "$lib/domain";

export interface SearchPostsResponse {
    posts: Post[];
    total_pages: number;
}