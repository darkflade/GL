import type { PlaylistsRepository } from "$lib/application/ports/playlists-repository";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";

export const searchPlaylists = (repo: PlaylistsRepository, query: SearchPostsQuery) => {
    return repo.searchPlaylists(query);
};
