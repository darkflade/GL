import type { Post } from "$lib/domain/models/post";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type { UUID } from "$lib/domain";
import type { SearchPostsResponse } from "$lib/infrastructure/repositories/dto";

export interface PostsRepository {
    searchPosts(query: SearchPostsQuery): Promise<SearchPostsResponse>;
    getPostByID(id: UUID): Promise<Post>;
}
