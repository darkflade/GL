import { api } from "$lib/infrastructure/http/client";
import type { PostsRepository } from "$lib/application/ports/posts-repository";
import type { Post } from "$lib/domain/models/post";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type { UUID } from "$lib/domain";
import type { SearchPostsResponse } from "$lib/infrastructure/repositories/dto";

export const postsHttpRepository: PostsRepository = {
    searchPosts: (query: SearchPostsQuery) => {
        const cursor =
            query.cursor.mode === "keyset"
                ? {
                      mode: query.cursor.mode,
                      last_id: query.cursor.last_id,
                      last_score: query.cursor.last_score,
                      limit: query.cursor.limit,
                  }
                : {
                      mode: query.cursor.mode,
                      page: query.cursor.page,
                      page_size: query.cursor.page_size,
                  };

        return api.post<SearchPostsResponse>("/posts/search", {
            tag_query: {
                must: query.tag_query.must,
                should: query.tag_query.should,
                must_not: query.tag_query.must_not,
            },
            text_query: query.text_query,
            cursor,
        });
    },
    getPostByID(id: UUID): Promise<Post> {
        return api.get<Post>(`/posts/${id}`)
    }
};
