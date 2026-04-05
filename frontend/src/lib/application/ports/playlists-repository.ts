import type { Playlist } from "$lib/domain/models/playlist";
import type { NewPlaylist, SearchPostsQuery, UpdatePlaylist, UUID } from "$lib/domain";
import type { SearchPlaylistsResponse } from "$lib/infrastructure/repositories/dto";

export interface PlaylistsRepository {
    searchPlaylists(query: SearchPostsQuery): Promise<SearchPlaylistsResponse>;
    getPlaylistByID(id: UUID): Promise<Playlist>;
    createPlaylist(payload: NewPlaylist): Promise<UUID>;
    updatePlaylist(id: UUID, payload: UpdatePlaylist): Promise<void>;
}
