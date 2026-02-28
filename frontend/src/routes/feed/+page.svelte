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
    import type { SearchPostsQuery } from "$lib/domain";
    import type { NextKeysetCursor } from "$lib/infrastructure/repositories/dto";

    const PAGINATION_MODE_KEY = "gl.pagination.mode";
    const KEYSET_LIMIT_KEY = "gl.pagination.keyset.limit";

    type KeysetCursor = Extract<SearchPostsQuery["cursor"], { mode: "keyset" }>;

    let posts = $state<Post[]>([]);
    let loading = $state(false);
    let textSearchValue = $state("");
    let paginationMode = $state<PaginationMode>("keyset");
    let storageReady = $state(false);
    let currentPage = $state(0);
    let totalPages = $state(0);
    let hasNext = $state(false);
    let nextCursor = $state<NextKeysetCursor | null>(null);
    let keysetLimit = $state(20);
    let keysetLimitInput = $state("20");
    let settingsOpen = $state(false);

    let keysetHistory = $state<KeysetCursor[]>([]);
    let keysetHistoryIndex = $state(-1);
    let keysetQueryKey = $state("");

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

        storageReady = true;
    });

    function sameKeysetCursor(a: KeysetCursor, b: KeysetCursor): boolean {
        return a.last_id === b.last_id && a.last_score === b.last_score && a.limit === b.limit;
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

    function updateKeysetHistory(filters: SearchPostsQuery) {
        if (filters.cursor.mode !== "keyset") {
            return;
        }

        const queryKey = JSON.stringify({
            tag_query: filters.tag_query,
            text_query: filters.text_query,
        });

        const cursor: KeysetCursor = {
            mode: "keyset",
            last_id: filters.cursor.last_id,
            last_score: filters.cursor.last_score,
            limit: filters.cursor.limit ?? keysetLimit,
        };

        if (queryKey !== keysetQueryKey) {
            keysetQueryKey = queryKey;
            keysetHistory = [cursor];
            keysetHistoryIndex = 0;
            return;
        }

        const existingIndex = keysetHistory.findIndex((item) => sameKeysetCursor(item, cursor));
        if (existingIndex >= 0) {
            keysetHistoryIndex = existingIndex;
            return;
        }

        const base = keysetHistoryIndex >= 0 ? keysetHistory.slice(0, keysetHistoryIndex + 1) : [];
        keysetHistory = [...base, cursor];
        keysetHistoryIndex = base.length;
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
        updateKeysetHistory(filters);
        fetchData(filters);
    });

    async function fetchData(filters: SearchPostsQuery) {
        loading = true;
        try {
            const serverResponse = await searchPosts(repositories.posts, filters);
            posts = serverResponse.posts;

            if ("total_pages" in serverResponse) {
                totalPages = serverResponse.total_pages;
                hasNext = false;
                nextCursor = null;
            } else {
                totalPages = 0;
                hasNext = serverResponse.has_next;
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
        if (keysetHistoryIndex <= 0) return;

        const previousCursor = keysetHistory[keysetHistoryIndex - 1];
        const query: SearchPostsQuery = {
            ...currentFilters,
            cursor: previousCursor,
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
            <button class="pager-btn" type="button" onclick={loadPrevByKeyset} disabled={keysetHistoryIndex <= 0}>
                Back
            </button>
            <button class="pager-btn" type="button" onclick={loadNextByKeyset} disabled={!hasNext}>
                Next
            </button>
            <span class="pager-label">Cursor {Math.max(1, keysetHistoryIndex + 1)}</span>
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
                    <a href="/post?id={post.id}">
                        <PostCard post={post} size={null} />
                    </a>
                {/each}
            </div>
        {/if}
    </main>
</div>

<style>
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
        gap: 1rem;
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
</style>
