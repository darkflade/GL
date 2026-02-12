import { postsHttpRepository } from "$lib/infrastructure/repositories/posts-http-repository";
import { tagsHttpRepository } from "$lib/infrastructure/repositories/tags-http-repository";
import { playlistsHttpRepository } from "$lib/infrastructure/repositories/playlists-http-repository";
import {authHttpRepository} from "$lib/infrastructure/repositories/auth-http-repository";

export const repositories = {
    posts: postsHttpRepository,
    tags: tagsHttpRepository,
    playlists: playlistsHttpRepository,
    authentication: authHttpRepository,
};
