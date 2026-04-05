import type { PostsRepository } from "$lib/application/ports/posts-repository";
import type { CreatePostInput } from "$lib/domain/value-objects/create";

export const createPost = (repo: PostsRepository, input: CreatePostInput) => {
    return repo.createPost(input);
};
