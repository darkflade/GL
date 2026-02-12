import { api } from "$lib/infrastructure/http/client";
import type { PlaylistsRepository } from "$lib/application/ports/playlists-repository";
import type { PlaylistSummary } from "$lib/domain/models/playlist";

export const playlistsHttpRepository: PlaylistsRepository = {
    searchPlaylists: (query: string) => {
        return api.get<PlaylistSummary[]>(`/playlists/search?query=${query}`);
    },
};
