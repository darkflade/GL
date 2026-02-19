import { api } from "$lib/infrastructure/http/client";
import type { PostsRepository } from "$lib/application/ports/posts-repository";
import type { Post } from "$lib/domain/models/post";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type {UUID} from "$lib/domain";

export const postsHttpRepository: PostsRepository = {
    searchPosts: (query: SearchPostsQuery) => {
        return api.post<Post[]>("/posts/search", {
            must: query.must,
            should: query.should,
            must_not: query.must_not,
        });
    },
    getPostByID(id: UUID): Promise<Post> {
        return api.get<Post>(`/posts/${id}`)
    }
};
