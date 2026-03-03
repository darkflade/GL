import type { PlaylistsRepository } from "$lib/application/ports/playlists-repository";
import type { UUID } from "$lib/domain";

export const getPlaylist = (repo: PlaylistsRepository, id: UUID) => {
    return repo.getPlaylistByID(id);
};
