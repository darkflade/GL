import type { PlaylistsRepository } from "$lib/application/ports/playlists-repository";
import type { UpdatePlaylist, UUID } from "$lib/domain";

export const updatePlaylist = (repo: PlaylistsRepository, id: UUID, payload: UpdatePlaylist) => {
    return repo.updatePlaylist(id, payload);
};
