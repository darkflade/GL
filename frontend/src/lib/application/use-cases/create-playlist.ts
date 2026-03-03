import type { PlaylistsRepository } from "$lib/application/ports/playlists-repository";
import type { NewPlaylist } from "$lib/domain";

export const createPlaylist = (repo: PlaylistsRepository, payload: NewPlaylist) => {
    return repo.createPlaylist(payload);
};
