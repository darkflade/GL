import type { Post } from "$lib/domain/models/post";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type {UUID} from "$lib/domain";

export interface PostsRepository {
    searchPosts(query: SearchPostsQuery): Promise<Post[]>;
    getPostByID(id: UUID): Promise<Post>;
}
