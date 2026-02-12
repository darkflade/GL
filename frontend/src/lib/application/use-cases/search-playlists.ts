import type { PlaylistsRepository } from "$lib/application/ports/playlists-repository";

export const searchPlaylists = (repo: PlaylistsRepository, query: string) => {
    return repo.searchPlaylists(query);
};
