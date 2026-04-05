<script lang="ts">
    import { repositories } from "$lib/composition/repositories";
    import { searchPlaylists } from "$lib/application/use-cases/search-playlists";
    import { updatePlaylist } from "$lib/application/use-cases/update-playlist";
    import { createPlaylist } from "$lib/application/use-cases/create-playlist";
    import { getPost } from "$lib/application/use-cases/get-post";
    import type { KeysetCursorDto } from "$lib/infrastructure/repositories/dto";
    import type { PlaylistSummary } from "$lib/domain/models/playlist";
    import type { SearchPostsQuery, UUID } from "$lib/domain";

    let {
        open = false,
        postIds = [] as UUID[],
        onClose,
        onDone,
    } = $props<{
        open?: boolean;
        postIds: UUID[];
        onClose: () => void;
        onDone?: () => void;
    }>();

    const SEARCH_LIMIT = 100;

    let loadingPlaylists = $state(false);
    let saving = $state(false);
    let errorText = $state("");
    let playlists = $state<PlaylistSummary[]>([]);
    let targetMode = $state<"existing" | "new">("existing");
    let selectedPlaylistId = $state<UUID | "">("");
    let newTitle = $state("");
    let newDescription = $state("");

    $effect(() => {
        if (!open) return;
        errorText = "";
        targetMode = "existing";
        selectedPlaylistId = "";
        void loadPlaylists();
    });

    function uniquePostIds(): UUID[] {
        const ids = postIds as UUID[];
        return [...new Set(ids)];
    }

    function buildSearchQuery(cursor?: KeysetCursorDto): SearchPostsQuery {
        return {
            tag_query: {
                must: [],
                should: [],
                must_not: [],
            },
            text_query: "",
            cursor: cursor
                ? {
                      mode: "keyset",
                      direction: cursor.direction,
                      last_id: cursor.last_id,
                      last_score: cursor.last_score,
                      limit: cursor.limit,
                  }
                : {
                      mode: "keyset",
                      limit: SEARCH_LIMIT,
                  },
        };
    }

    async function loadPlaylists() {
        loadingPlaylists = true;
        errorText = "";
        try {
            const all: PlaylistSummary[] = [];
            let nextCursor: KeysetCursorDto | undefined;
            let pageGuard = 20;
            while (pageGuard > 0) {
                const response = await searchPlaylists(repositories.playlists, buildSearchQuery(nextCursor));
                all.push(...response.playlists);
                if (!response.has_next || !response.next_cursor) {
                    break;
                }
                nextCursor = response.next_cursor;
                pageGuard -= 1;
            }

            const byId = new Map<UUID, PlaylistSummary>();
            for (const item of all) {
                byId.set(item.id, item);
            }

            playlists = [...byId.values()];
        } catch (error) {
            console.error(error);
            errorText = "Failed to load playlists";
            playlists = [];
        } finally {
            loadingPlaylists = false;
        }
    }

    async function mergeTagIds(ids: UUID[]): Promise<UUID[]> {
        const fetchedPosts = await Promise.all(ids.map((id) => getPost(repositories.posts, id).catch(() => null)));
        const tagSet = new Set<UUID>();
        for (const post of fetchedPosts) {
            if (!post) continue;
            for (const tag of post.tags) {
                tagSet.add(tag.id);
            }
        }
        return [...tagSet];
    }

    async function applyToExistingPlaylist(playlistId: UUID, ids: UUID[]) {
        const itemEvents = [...ids].reverse().map((postId) => ({
            op: "add" as const,
            after_id: null,
            content: {
                type: "post" as const,
                post_id: postId,
            },
        }));

        await updatePlaylist(repositories.playlists, playlistId, {
            item_events: itemEvents,
        });
    }

    async function createNewPlaylist(ids: UUID[]) {
        const title = newTitle.trim();
        const tagIds = await mergeTagIds(ids);
        const items = ids.map((postId, index) => ({
            position: index + 1,
            content: {
                type: "post" as const,
                post_id: postId,
            },
        }));

        await createPlaylist(repositories.playlists, {
            title,
            description: newDescription.trim() ? newDescription.trim() : null,
            tag_ids: tagIds.length > 0 ? tagIds : null,
            items,
        });
    }

    async function submit() {
        const ids = uniquePostIds();
        if (ids.length === 0) {
            errorText = "No posts selected";
            return;
        }

        saving = true;
        errorText = "";
        try {
            if (targetMode === "existing") {
                if (!selectedPlaylistId) {
                    errorText = "Select a playlist";
                    return;
                }
                await applyToExistingPlaylist(selectedPlaylistId, ids);
            } else {
                if (!newTitle.trim()) {
                    errorText = "Title is required";
                    return;
                }
                await createNewPlaylist(ids);
            }

            onDone?.();
            onClose();
        } catch (error) {
            console.error(error);
            errorText = "Failed to apply playlist changes";
        } finally {
            saving = false;
        }
    }
</script>

{#if open}
    <button class="backdrop" type="button" aria-label="Close dialog" onclick={onClose}></button>
    <div class="dialog" role="dialog" aria-modal="true" aria-label="Add to playlist">
        <div class="head">
            <h3>Add To Playlist</h3>
            <button type="button" class="close-btn" onclick={onClose}>Close</button>
        </div>

        <p class="meta">Selected posts: {uniquePostIds().length}</p>

        <div class="mode-row">
            <button
                type="button"
                class:active={targetMode === "existing"}
                class="mode-btn"
                onclick={() => (targetMode = "existing")}
            >
                Existing
            </button>
            <button
                type="button"
                class:active={targetMode === "new"}
                class="mode-btn"
                onclick={() => (targetMode = "new")}
            >
                Add To New
            </button>
        </div>

        {#if targetMode === "existing"}
            {#if loadingPlaylists}
                <p class="meta">Loading playlists...</p>
            {:else if playlists.length === 0}
                <p class="meta">No playlists found</p>
            {:else}
                <div class="playlists-list">
                    {#each playlists as playlist (playlist.id)}
                        <button
                            type="button"
                            class:active={selectedPlaylistId === playlist.id}
                            class="playlist-item"
                            onclick={() => (selectedPlaylistId = playlist.id)}
                        >
                            <span class="title">{playlist.title}</span>
                            <span class="count">{playlist.item_count} items</span>
                        </button>
                    {/each}
                </div>
            {/if}
        {:else}
            <div class="field">
                <label for="playlist-title">Title</label>
                <input id="playlist-title" class="input" bind:value={newTitle} />
            </div>
            <div class="field">
                <label for="playlist-description">Description</label>
                <textarea id="playlist-description" class="input" rows="3" bind:value={newDescription}></textarea>
            </div>
        {/if}

        {#if errorText}
            <p class="error">{errorText}</p>
        {/if}

        <div class="actions">
            <button type="button" class="btn" onclick={onClose} disabled={saving}>Cancel</button>
            <button type="button" class="btn primary" onclick={submit} disabled={saving}>
                {saving ? "Saving..." : "Confirm"}
            </button>
        </div>
    </div>
{/if}

<style>
    .backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        border: 0;
        z-index: 60;
    }

    .dialog {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: min(520px, calc(100vw - 2rem));
        max-height: calc(100vh - 2rem);
        overflow: auto;
        border: 1px solid #e5e7eb;
        border-radius: 12px;
        background: #fff;
        padding: 1rem;
        z-index: 70;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .head {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .head h3 {
        margin: 0;
    }

    .meta {
        margin: 0;
        color: #6b7280;
    }

    .close-btn,
    .btn,
    .mode-btn,
    .playlist-item {
        border: 1px solid #d1d5db;
        background: #fff;
        border-radius: 8px;
        padding: 0.4rem 0.7rem;
        cursor: pointer;
    }

    .mode-row {
        display: flex;
        gap: 0.5rem;
    }

    .mode-btn.active {
        border-color: #2563eb;
        color: #2563eb;
    }

    .playlists-list {
        display: flex;
        flex-direction: column;
        gap: 0.45rem;
        max-height: 280px;
        overflow: auto;
    }

    .playlist-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        text-align: left;
    }

    .playlist-item.active {
        border-color: #2563eb;
        background: #eff6ff;
    }

    .playlist-item .title {
        font-weight: 600;
    }

    .playlist-item .count {
        color: #6b7280;
        font-size: 0.85rem;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .input {
        border: 1px solid #d1d5db;
        border-radius: 8px;
        padding: 0.45rem 0.6rem;
    }

    .error {
        margin: 0;
        color: #b91c1c;
    }

    .actions {
        display: flex;
        gap: 0.5rem;
        justify-content: flex-end;
    }

    .btn.primary {
        border-color: #2563eb;
        background: #2563eb;
        color: #fff;
    }

    .btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
