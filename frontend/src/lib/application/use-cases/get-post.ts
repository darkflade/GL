import type {PostsRepository} from "$lib/application";
import type {UUID} from "$lib/domain";

export const getPost = (repo: PostsRepository, id: UUID) => {
    return repo.getPostByID(id);
};