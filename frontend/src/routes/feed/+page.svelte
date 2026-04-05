<script lang="ts">
    import { onMount } from "svelte";
    import type { Post } from "$lib/domain/models/post";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { searchPosts } from "$lib/application/use-cases/search-posts";
    import { repositories } from "$lib/composition/repositories";
    import PostCard from "$lib/features/feed/components/PostCard.svelte";
    import PostSearchControls from "$lib/features/feed/components/PostSearchControls.svelte";
    import {
        applyPaginationMode,
        buildSearchHref,
        queryFromUrl,
        type PaginationMode,
    } from "$lib/features/feed/search-query";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import { toSearchInput } from "$lib/utils/search";
    import AddToPlaylistDialog from "$lib/features/playlists/components/AddToPlaylistDialog.svelte";
    import { readSelectedPostIds, writeSelectedPostIds } from "$lib/infrastructure/storage/selected-posts";
    import type { SearchPostsQuery, UUID } from "$lib/domain";
    import type { KeysetCursorDto } from "$lib/infrastructure/repositories/dto";

    const PAGINATION_MODE_KEY = "gl.pagination.mode";
    const KEYSET_LIMIT_KEY = "gl.pagination.keyset.limit";

    let posts = $state<Post[]>([]);
    let loading = $state(false);
    let textSearchValue = $state("");
    let paginationMode = $state<PaginationMode>("keyset");
    let storageReady = $state(false);
    let currentPage = $state(0);
    let totalPages = $state(0);
    let hasPrev = $state(false);
    let hasNext = $state(false);
    let prevCursor = $state<KeysetCursorDto | null>(null);
    let nextCursor = $state<KeysetCursorDto | null>(null);
    let keysetLimit = $state(20);
    let keysetLimitInput = $state("20");
    let settingsOpen = $state(false);
    let selectionMode = $state(false);
    let selectedPostIds = $state<UUID[]>([]);
    let playlistDialogOpen = $state(false);

    let currentFilters = $state<SearchPostsQuery>({
        tag_query: {
            must: [],
            should: [],
            must_not: [],
        },
        text_query: "",
        cursor: { mode: "keyset", limit: 20 },
    });

    onMount(() => {
        const savedMode = localStorage.getItem(PAGINATION_MODE_KEY);
        if (savedMode === "offset" || savedMode === "keyset") {
            paginationMode = savedMode;
        }

        const savedLimit = Number.parseInt(localStorage.getItem(KEYSET_LIMIT_KEY) ?? "20", 10);
        if (Number.isFinite(savedLimit) && savedLimit > 0) {
            keysetLimit = savedLimit;
            keysetLimitInput = String(savedLimit);
        }

        void restoreSelectedPosts();
        storageReady = true;
    });

    async function restoreSelectedPosts() {
        selectedPostIds = await readSelectedPostIds();
    }

    function isPostSelected(postId: UUID): boolean {
        return selectedPostIds.includes(postId);
    }

    async function setSelectedPostIds(ids: UUID[]) {
        selectedPostIds = ids;
        await writeSelectedPostIds(ids);
    }

    async function togglePostSelection(postId: UUID) {
        const next = isPostSelected(postId)
            ? selectedPostIds.filter((id) => id !== postId)
            : [...selectedPostIds, postId];
        await setSelectedPostIds(next);
    }

    async function clearPostSelection() {
        await setSelectedPostIds([]);
    }

    function openPlaylistSelectionMode() {
        selectionMode = true;
    }

    function closePlaylistSelectionMode() {
        selectionMode = false;
    }

    function openAddToPlaylistDialog() {
        if (selectedPostIds.length === 0) return;
        playlistDialogOpen = true;
    }

    function closeAddToPlaylistDialog() {
        playlistDialogOpen = false;
    }

    function handlePlaylistApplied() {
        playlistDialogOpen = false;
        selectionMode = false;
        void clearPostSelection();
    }

    function normalizeQuery(query: SearchPostsQuery, mode: PaginationMode): SearchPostsQuery {
        const withMode = applyPaginationMode(query, mode);
        if (withMode.cursor.mode === "keyset") {
            return {
                ...withMode,
                cursor: {
                    ...withMode.cursor,
                    mode: "keyset",
                    limit: keysetLimit,
                },
            };
        }
        return withMode;
    }

    $effect(() => {
        if (!storageReady) {
            return;
        }

        const fromUrl = queryFromUrl(page.url.searchParams);
        const hasModeInUrl = page.url.searchParams.has("mode");
        const mode = hasModeInUrl ? fromUrl.cursor.mode : paginationMode;
        const filters = normalizeQuery(fromUrl, mode);

        const hasLimitInUrl = mode === "keyset" && page.url.searchParams.has("limit");
        if (!hasModeInUrl || (mode === "keyset" && !hasLimitInUrl)) {
            const newLink = buildSearchHref(page.url.pathname, filters);
            goto(newLink, {
                keepFocus: true,
                replaceState: true,
                noScroll: true,
            });
            return;
        }

        if (paginationMode !== mode) {
            paginationMode = mode;
            localStorage.setItem(PAGINATION_MODE_KEY, mode);
        }

        currentFilters = filters;
        currentPage = filters.cursor.mode === "offset" ? filters.cursor.page : 0;
        textSearchValue = toSearchInput(filters);
        fetchData(filters);
    });

    async function fetchData(filters: SearchPostsQuery) {
        loading = true;
        try {
            const serverResponse = await searchPosts(repositories.posts, filters);
            posts = serverResponse.posts;

            if ("total_pages" in serverResponse) {
                totalPages = serverResponse.total_pages;
                hasPrev = false;
                hasNext = false;
                prevCursor = null;
                nextCursor = null;
            } else {
                totalPages = 0;
                hasPrev = serverResponse.has_prev;
                hasNext = serverResponse.has_next;
                prevCursor = serverResponse.prev_cursor ?? null;
                nextCursor = serverResponse.next_cursor ?? null;
            }
        } catch (error) {
            console.error(error);
        } finally {
            loading = false;
        }
    }

    async function handleSearchQuery(query: SearchPostsQuery) {
        const finalQuery = normalizeQuery(query, paginationMode);
        const newLink = buildSearchHref(page.url.pathname, finalQuery);

        await goto(newLink, {
            keepFocus: true,
            replaceState: false,
            noScroll: true,
        });
    }

    async function changePage(nextPage: number) {
        const query: SearchPostsQuery = {
            ...currentFilters,
            cursor: { mode: "offset", page: Math.max(0, nextPage) },
        };
        await handleSearchQuery(query);
    }

    async function loadNextByKeyset() {
        if (!nextCursor) return;

        const query: SearchPostsQuery = {
            ...currentFilters,
            cursor: {
                ...nextCursor,
                mode: "keyset",
                limit: keysetLimit,
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
                limit: keysetLimit,
            },
        };
        await handleSearchQuery(query);
    }

    async function changePaginationMode(nextMode: PaginationMode) {
        paginationMode = nextMode;
        localStorage.setItem(PAGINATION_MODE_KEY, nextMode);
        const query = normalizeQuery(currentFilters, nextMode);
        await handleSearchQuery(query);
    }

    async function applyKeysetLimit() {
        const parsed = Number.parseInt(keysetLimitInput, 10);
        if (!Number.isFinite(parsed) || parsed <= 0) {
            keysetLimitInput = String(keysetLimit);
            return;
        }

        keysetLimit = parsed;
        localStorage.setItem(KEYSET_LIMIT_KEY, String(parsed));

        if (paginationMode === "keyset") {
            const query: SearchPostsQuery = {
                ...currentFilters,
                cursor: {
                    mode: "keyset",
                    limit: parsed,
                },
            };
            await handleSearchQuery(query);
        }
    }
</script>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <Header/>
    <header class="bg-white sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <h1 class="text-xl font-bold tracking-tight">Glab Storage</h1>
        <div class="header-actions">
            <PostSearchControls
                value={textSearchValue}
                paginationMode={paginationMode}
                onQueryChange={handleSearchQuery}
            />
            {#if selectionMode}
                <span class="selection-count">Selected: {selectedPostIds.length}</span>
                <button class="settings-btn" type="button" onclick={openAddToPlaylistDialog} disabled={selectedPostIds.length === 0}>
                    Add To Playlist
                </button>
                <button class="settings-btn" type="button" onclick={() => void clearPostSelection()} disabled={selectedPostIds.length === 0}>
                    Clear
                </button>
                <button class="settings-btn" type="button" onclick={closePlaylistSelectionMode}>
                    Exit Select
                </button>
            {:else}
                <button class="settings-btn" type="button" onclick={openPlaylistSelectionMode}>
                    Select For Playlist
                </button>
            {/if}
            <button class="settings-btn" type="button" onclick={() => (settingsOpen = true)}>Settings</button>
        </div>
    </header>

    {#if settingsOpen}
        <button class="drawer-backdrop" aria-label="Close settings" onclick={() => (settingsOpen = false)}></button>
        <div class="drawer" role="dialog" aria-modal="true" aria-label="Feed settings">
            <div class="drawer-head">
                <h2>Feed Settings</h2>
                <button class="drawer-close" type="button" onclick={() => (settingsOpen = false)}>Close</button>
            </div>

            <div class="drawer-field">
                <label for="pagination-mode">Pagination mode</label>
                <select
                    id="pagination-mode"
                    class="drawer-select"
                    value={paginationMode}
                    onchange={(event) => changePaginationMode((event.currentTarget as HTMLSelectElement).value as PaginationMode)}
                >
                    <option value="keyset">keyset</option>
                    <option value="offset">offset</option>
                </select>
            </div>

            <div class="drawer-field">
                <label for="keyset-limit">Keyset limit</label>
                <input
                    id="keyset-limit"
                    class="drawer-input"
                    type="number"
                    min="1"
                    bind:value={keysetLimitInput}
                />
                <button type="button" class="drawer-apply" onclick={applyKeysetLimit}>Apply</button>
            </div>
        </div>
    {/if}

    <div class="pager">
        {#if paginationMode === "offset"}
            <button class="pager-btn" type="button" onclick={() => changePage(currentPage - 1)} disabled={currentPage <= 0}>
                Prev
            </button>
            <span class="pager-label">Page {currentPage + 1}</span>
            <button class="pager-btn" type="button" onclick={() => changePage(currentPage + 1)} disabled={currentPage >= Math.max(0, totalPages - 1)}>
                Next
            </button>
        {:else}
            <button class="pager-btn" type="button" onclick={loadPrevByKeyset} disabled={!hasPrev}>
                Back
            </button>
            <button class="pager-btn" type="button" onclick={loadNextByKeyset} disabled={!hasNext}>
                Next
            </button>
        {/if}
    </div>

    <main>
        {#if loading}
            <div class="flex items-center justify-center h-64">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
            </div>
        {:else if posts.length === 0}
            <EmptyList/>
        {:else}
            <div class="grid">
                {#each posts as post (post.id)}
                    <div class="post-slot" class:selected={isPostSelected(post.id)}>
                        {#if selectionMode}
                            <label class="checkbox-corner">
                                <input
                                    type="checkbox"
                                    checked={isPostSelected(post.id)}
                                    onclick={(event) => event.stopPropagation()}
                                    onchange={() => void togglePostSelection(post.id)}
                                />
                            </label>
                            <button class="select-card-btn" type="button" onclick={() => void togglePostSelection(post.id)}>
                                <PostCard post={post} size={null} />
                            </button>
                        {:else}
                            <a href="/post?id={post.id}">
                                <PostCard post={post} size={null} />
                            </a>
                        {/if}
                    </div>
                {/each}
            </div>
        {/if}
    </main>
</div>

<AddToPlaylistDialog
    open={playlistDialogOpen}
    postIds={selectedPostIds}
    onClose={closeAddToPlaylistDialog}
    onDone={handlePlaylistApplied}
/>

<style>
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
        gap: 1rem;
        padding: 1rem;
    }

    h1 {
        color: #8e8e8f;
        font-family: "Symbola";
    }

    .header-actions {
        margin-left: auto;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        min-width: 0;
        flex: 1;
    }

    .settings-btn {
        border: 1px solid #d1d5db;
        background: #fff;
        border-radius: 8px;
        padding: 0.45rem 0.75rem;
        cursor: pointer;
        white-space: nowrap;
    }

    .selection-count {
        color: #111827;
        font-weight: 600;
        white-space: nowrap;
    }

    .drawer-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        border: 0;
        z-index: 40;
    }

    .drawer {
        position: fixed;
        top: 0;
        right: 0;
        width: min(360px, 100vw);
        height: 100vh;
        background: #fff;
        border-left: 1px solid #e5e7eb;
        box-shadow: -8px 0 24px rgba(0, 0, 0, 0.12);
        padding: 1rem;
        z-index: 50;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .drawer-head {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .drawer-head h2 {
        margin: 0;
        font-size: 1rem;
    }

    .drawer-close {
        border: 1px solid #d1d5db;
        background: #fff;
        border-radius: 8px;
        padding: 0.35rem 0.6rem;
        cursor: pointer;
    }

    .drawer-field {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .drawer-select,
    .drawer-input {
        border: 1px solid #d1d5db;
        border-radius: 8px;
        padding: 0.45rem 0.55rem;
        background: #fff;
    }

    .drawer-apply {
        width: fit-content;
        border: 1px solid #d1d5db;
        background: #f9fafb;
        border-radius: 8px;
        padding: 0.35rem 0.75rem;
        cursor: pointer;
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

    .pager-label {
        color: #374151;
        font-weight: 500;
    }

    .post-slot {
        position: relative;
        border-radius: 10px;
        overflow: hidden;
        border: 2px solid transparent;
    }

    .post-slot.selected {
        border-color: #2563eb;
        box-shadow: 0 0 0 1px rgba(37, 99, 235, 0.2);
    }

    .checkbox-corner {
        position: absolute;
        top: 0.45rem;
        right: 0.45rem;
        z-index: 5;
        background: rgba(255, 255, 255, 0.95);
        border-radius: 6px;
        padding: 0.2rem;
        border: 1px solid #d1d5db;
    }

    .checkbox-corner input {
        width: 1rem;
        height: 1rem;
        cursor: pointer;
    }

    .select-card-btn {
        display: block;
        width: 100%;
        background: transparent;
        border: 0;
        padding: 0;
        cursor: pointer;
    }
</style>
