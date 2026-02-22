import { api } from "$lib/infrastructure/http/client";
import type { PostsRepository } from "$lib/application/ports/posts-repository";
import type { Post } from "$lib/domain/models/post";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type {UUID} from "$lib/domain";
import type {SearchPostsResponse} from "$lib/infrastructure/repositories/dto";

export const postsHttpRepository: PostsRepository = {
    searchPosts: (query: SearchPostsQuery) => {
        return api.post<SearchPostsResponse>("/posts/search", {
            tag_query: {
                must: query.tag_query.must,
                should: query.tag_query.should,
                must_not: query.tag_query.must_not,
            },
            cursor: query.cursor,
        });
    },
    getPostByID(id: UUID): Promise<Post> {
        return api.get<Post>(`/posts/${id}`)
    }
};
