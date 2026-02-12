import type { PostsRepository } from "$lib/application/ports/posts-repository";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";

export const searchPosts = (repo: PostsRepository, query: SearchPostsQuery) => {
    return repo.searchPosts(query);
};
