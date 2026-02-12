import type { Post } from "$lib/domain/models/post";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";

export interface PostsRepository {
    searchPosts(query: SearchPostsQuery): Promise<Post[]>;
}
