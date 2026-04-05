import { api } from "$lib/infrastructure/http/client";
import type { PlaylistsRepository } from "$lib/application/ports/playlists-repository";
import type { Playlist } from "$lib/domain/models/playlist";
import type { NewPlaylist, SearchPostsQuery, UpdatePlaylist, UUID } from "$lib/domain";
import type { SearchPlaylistsResponse } from "$lib/infrastructure/repositories/dto";

export const playlistsHttpRepository: PlaylistsRepository = {
    searchPlaylists: (query: SearchPostsQuery) => {
        const cursor =
            query.cursor.mode === "keyset"
                ? {
                    mode: "keyset" as const,
                    direction: query.cursor.direction,
                    last_id: query.cursor.last_id,
                    last_score: query.cursor.last_score,
                    limit: query.cursor.limit ?? 20,
                }
                : {
                    mode: "keyset" as const,
                    limit: query.cursor.page_size ?? 20,
                };

        return api.post<SearchPlaylistsResponse>("/playlists/search", {
            tag_query: {
                must: query.tag_query.must,
                should: query.tag_query.should,
                must_not: query.tag_query.must_not,
            },
            text_query: query.text_query,
            cursor,
        });
    },
    getPlaylistByID(id: UUID): Promise<Playlist> {
        return api.get<Playlist>(`/playlists/${id}`);
    },
    createPlaylist(payload: NewPlaylist): Promise<UUID> {
        return api.post<UUID>("/playlists", payload);
    },
    updatePlaylist(id: UUID, payload: UpdatePlaylist): Promise<void> {
        return api.patch<void>(`/playlists/${id}`, payload);
    },
};
