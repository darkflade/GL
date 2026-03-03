<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { repositories } from "$lib/composition/repositories";
    import { searchPlaylists } from "$lib/application/use-cases/search-playlists";
    import { getPlaylist } from "$lib/application/use-cases/get-playlist";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import PostSearchControls from "$lib/features/feed/components/PostSearchControls.svelte";
    import {
        applyPaginationMode,
        buildSearchHref,
        queryFromUrl
    } from "$lib/features/feed/search-query";
    import { serializeQuery, toSearchInput } from "$lib/utils/search";
    import type { KeysetCursorDto } from "$lib/infrastructure/repositories/dto";
    import type { Playlist, PlaylistSummary } from "$lib/domain/models/playlist";
    import type { SearchPostsQuery, UUID } from "$lib/domain";

    const KEYSET_LIMIT = 20;

    let loading = $state(false);
    let textSearchValue = $state("");
    let playlists = $state<PlaylistSummary[]>([]);
    let currentFilters = $state<SearchPostsQuery>({
        tag_query: {
            must: [],
            should: [],
            must_not: [],
        },
        text_query: "",
        cursor: { mode: "keyset", limit: KEYSET_LIMIT },
    });

    let hasPrev = $state(false);
    let hasNext = $state(false);
    let prevCursor = $state<KeysetCursorDto | null>(null);
    let nextCursor = $state<KeysetCursorDto | null>(null);

    let selectedPlaylistId = $state<UUID | null>(null);
    let selectedPlaylist = $state<Playlist | null>(null);

    function normalizeKeysetQuery(query: SearchPostsQuery): SearchPostsQuery {
        const withMode = applyPaginationMode(query, "keyset");
        const keysetCursor =
            withMode.cursor.mode === "keyset"
                ? withMode.cursor
                : { mode: "keyset" as const };

        return {
            ...withMode,
            cursor: {
                ...keysetCursor,
                mode: "keyset",
                limit: keysetCursor.limit ?? KEYSET_LIMIT,
            },
        };
    }

    $effect(() => {
        const playlistId = page.url.searchParams.get("id");
        if (playlistId) {
            selectedPlaylistId = playlistId;
            fetchPlaylist(playlistId);
            return;
        }

        selectedPlaylistId = null;
        selectedPlaylist = null;

        const fromUrl = queryFromUrl(page.url.searchParams);
        const filters = normalizeKeysetQuery(fromUrl);

        const hasModeInUrl = page.url.searchParams.get("mode") === "keyset";
        const hasLimitInUrl = page.url.searchParams.has("limit");
        if (!hasModeInUrl || !hasLimitInUrl) {
            const newLink = buildSearchHref(page.url.pathname, filters);
            goto(newLink, {
                keepFocus: true,
                replaceState: true,
                noScroll: true,
            });
            return;
        }

        currentFilters = filters;
        textSearchValue = toSearchInput(filters);
        fetchPlaylists(filters);
    });

    async function fetchPlaylists(filters: SearchPostsQuery) {
        loading = true;
        try {
            const response = await searchPlaylists(repositories.playlists, filters);
            playlists = response.playlists;
            hasPrev = response.has_prev;
            hasNext = response.has_next;
            prevCursor = response.prev_cursor ?? null;
            nextCursor = response.next_cursor ?? null;
        } catch (error) {
            console.error(error);
            playlists = [];
            hasPrev = false;
            hasNext = false;
            prevCursor = null;
            nextCursor = null;
        } finally {
            loading = false;
        }
    }

    async function fetchPlaylist(id: UUID) {
        loading = true;
        try {
            selectedPlaylist = await getPlaylist(repositories.playlists, id);
        } catch (error) {
            console.error(error);
            selectedPlaylist = null;
        } finally {
            loading = false;
        }
    }

    async function handleSearchQuery(query: SearchPostsQuery) {
        const finalQuery = normalizeKeysetQuery(query);
        const newLink = buildSearchHref(page.url.pathname, finalQuery);
        await goto(newLink, {
            keepFocus: true,
            replaceState: false,
            noScroll: true,
        });
    }

    async function loadNextByKeyset() {
        if (!nextCursor) return;

        const query: SearchPostsQuery = {
            ...currentFilters,
            cursor: {
                ...nextCursor,
                mode: "keyset",
                limit: KEYSET_LIMIT,
            },
        };
        await handleSearchQuery(query);
    }

    async function loadPrevByKeyset() {
        if (!prevCursor) return;

        const query: SearchPostsQuery = {
            ...currentFilters,
            cursor: {
                ...prevCursor,
                mode: "keyset",
                limit: KEYSET_LIMIT,
            },
        };
        await handleSearchQuery(query);
    }

    async function openPlaylist(id: UUID) {
        const params = new URLSearchParams(serializeQuery(currentFilters));
        params.set("id", id);
        await goto(`${page.url.pathname}?${params.toString()}`, {
            keepFocus: true,
            replaceState: false,
            noScroll: true,
        });
    }

    async function closePlaylistDetails() {
        const newLink = buildSearchHref(page.url.pathname, currentFilters);
        await goto(newLink, {
            keepFocus: true,
            replaceState: false,
            noScroll: true,
        });
    }
</script>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <Header />
    <header class="bg-white sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <h1 class="text-xl font-bold tracking-tight">Playlists</h1>
        <div class="header-actions">
            {#if !selectedPlaylistId}
                <PostSearchControls
                    value={textSearchValue}
                    paginationMode="keyset"
                    onQueryChange={handleSearchQuery}
                />
            {:else}
                <button class="pager-btn" type="button" onclick={closePlaylistDetails}>Back to list</button>
            {/if}
        </div>
    </header>

    {#if !selectedPlaylistId}
        <div class="pager">
            <button class="pager-btn" type="button" onclick={loadPrevByKeyset} disabled={!hasPrev}>
                Back
            </button>
            <button class="pager-btn" type="button" onclick={loadNextByKeyset} disabled={!hasNext}>
                Next
            </button>
        </div>
    {/if}

    <main>
        {#if loading}
            <div class="flex items-center justify-center h-64">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
            </div>
        {:else if selectedPlaylistId}
            {#if selectedPlaylist}
                <section class="playlist-details">
                    <h2>{selectedPlaylist.title}</h2>
                    {#if selectedPlaylist.description}
                        <p class="description">{selectedPlaylist.description}</p>
                    {/if}
                    <p class="meta">ID: {selectedPlaylist.id}</p>
                    <p class="meta">Items: {selectedPlaylist.items.length}</p>

                    <div class="tags">
                        {#each selectedPlaylist.tags as tag (tag.id)}
                            <span class="tag">{tag.name}</span>
                        {/each}
                    </div>

                    <div class="items">
                        {#each selectedPlaylist.items as item (item.id)}
                            <article class="item">
                                <div class="item-head">#{item.position}</div>
                                {#if "Post" in item.content}
                                    <a href="/post?id={item.content.Post.id}" class="item-link">{item.content.Post.title}</a>
                                {:else}
                                    <p class="note">{item.content.Note}</p>
                                {/if}
                            </article>
                        {/each}
                    </div>
                </section>
            {:else}
                <EmptyList />
            {/if}
        {:else if playlists.length === 0}
            <EmptyList />
        {:else}
            <div class="playlist-grid">
                {#each playlists as playlist (playlist.id)}
                    <button class="playlist-card" type="button" onclick={() => openPlaylist(playlist.id)}>
                        <h3>{playlist.title}</h3>
                        <p class="description">{playlist.description}</p>
                        <p class="meta">Items: {playlist.item_count}</p>
                        <div class="tags">
                            {#each playlist.tags as tag (tag.id)}
                                <span class="tag">{tag.name}</span>
                            {/each}
                        </div>
                    </button>
                {/each}
            </div>
        {/if}
    </main>
</div>

<style>
    .header-actions {
        margin-left: auto;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        min-width: 0;
        flex: 1;
        justify-content: flex-end;
    }

    .pager {
        padding: 0.75rem 1.5rem;
        display: flex;
        align-items: center;
        gap: 0.75rem;
        background: #f9fafb;
        border-bottom: 1px solid #e5e7eb;
    }

    .pager-btn {
        border: 1px solid #d1d5db;
        background: #fff;
        border-radius: 8px;
        padding: 0.35rem 0.75rem;
        cursor: pointer;
    }

    .pager-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .playlist-grid {
        padding: 1rem;
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
        gap: 0.8rem;
    }

    .playlist-card {
        border: 1px solid #e5e7eb;
        background: #fff;
        border-radius: 10px;
        text-align: left;
        padding: 0.8rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        cursor: pointer;
    }

    .playlist-card h3 {
        margin: 0;
        font-size: 1rem;
        font-weight: 700;
    }

    .description {
        margin: 0;
        color: #6b7280;
    }

    .meta {
        margin: 0;
        color: #374151;
        font-weight: 600;
    }

    .tags {
        display: flex;
        flex-wrap: wrap;
        gap: 0.35rem;
    }

    .tag {
        border: 1px solid #d1d5db;
        border-radius: 999px;
        padding: 0.1rem 0.45rem;
        font-size: 0.75rem;
        color: #374151;
    }

    .playlist-details {
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .playlist-details h2 {
        margin: 0;
    }

    .items {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        margin-top: 0.4rem;
    }

    .item {
        border: 1px solid #e5e7eb;
        border-radius: 8px;
        padding: 0.6rem 0.75rem;
        background: #fff;
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .item-head {
        font-weight: 700;
        color: #111827;
    }

    .item-link {
        color: #2563eb;
        text-decoration: none;
        width: fit-content;
    }

    .item-link:hover {
        text-decoration: underline;
    }

    .note {
        margin: 0;
        white-space: pre-wrap;
    }
</style>
