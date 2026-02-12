import type { PlaylistSummary } from "$lib/domain/models/playlist";

export interface PlaylistsRepository {
    searchPlaylists(query: string): Promise<PlaylistSummary[]>;
}
