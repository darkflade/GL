<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { repositories } from "$lib/composition/repositories";
    import { searchCollections } from "$lib/application/use-cases/search-collections";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import PostSearchControls from "$lib/features/feed/components/PostSearchControls.svelte";
    import { ApiError } from "$lib/infrastructure/http/client";
    import {
        applyPaginationMode,
        buildSearchHref,
        queryFromUrl
    } from "$lib/features/feed/search-query";
    import { toSearchInput } from "$lib/utils/search";
    import type { KeysetCursorDto } from "$lib/infrastructure/repositories/dto";
    import type { CollectionSummary } from "$lib/domain/models/collection";
    import type { SearchPostsQuery, UUID } from "$lib/domain";

    import CollectionDetailsDialog from "$lib/features/collections/components/CollectionDetailsDialog.svelte";
    let details = $state<{ collection: CollectionSummary | null; deleting: boolean } | null>(null);
    const KEYSET_LIMIT = 20;

    let loading = $state(false);
    let textSearchValue = $state("");
    let collections = $state<CollectionSummary[]>([]);
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
    let fetchError = $state("");
    let unauthorized = $state(false);

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
        fetchCollections(filters);
    });

    async function fetchCollections(filters: SearchPostsQuery) {
        loading = true;
        fetchError = "";
        unauthorized = false;
        try {
            const response = await searchCollections(repositories.collections, filters);
            collections = response.collections;
            hasPrev = response.has_prev;
            hasNext = response.has_next;
            prevCursor = response.prev_cursor ?? null;
            nextCursor = response.next_cursor ?? null;
        } catch (error) {
            console.error(error);
            collections = [];
            hasPrev = false;
            hasNext = false;
            prevCursor = null;
            nextCursor = null;
            if (error instanceof ApiError && error.code === 401) {
                unauthorized = true;
                fetchError = "Session expired or missing. Please sign in.";
            } else if (error instanceof Error) {
                fetchError = error.message;
            } else {
                fetchError = "Failed to load collections.";
            }
        } finally {
            loading = false;
        }
    }

    function getSignInHref(): string {
        const redirectTo = `${page.url.pathname}${page.url.search}`;
        return `/auth/login?redirect=${encodeURIComponent(redirectTo)}`;
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

    async function openCollection(id: UUID) {
        await goto(`/collection?id=${encodeURIComponent(id)}`, {
            keepFocus: true,
            replaceState: false,
            noScroll: true,
        });
    }
</script>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <Header />
    <header class="bg-white sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <h1 class="text-xl font-bold tracking-tight">Collections</h1>
        <div class="header-actions">
            <PostSearchControls
                sort={currentFilters.sort}
                value={textSearchValue}
                paginationMode="keyset"
                onQueryChange={handleSearchQuery}
            />
        </div>
    </header>

    <div class="pager">
        <button class="pager-btn" type="button" onclick={() => details = { collection: null, deleting: false }}>+ New collection</button>
        <button class="pager-btn" type="button" onclick={loadPrevByKeyset} disabled={!hasPrev}>
            Back
        </button>
        <button class="pager-btn" type="button" onclick={loadNextByKeyset} disabled={!hasNext}>
            Next
        </button>
    </div>

    <main>
        {#if loading}
            <div class="flex items-center justify-center h-64">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
            </div>
        {:else if fetchError}
            <section class="error-box">
                <p>{fetchError}</p>
                {#if unauthorized}
                    <a class="login-link" href={getSignInHref()}>Sign In</a>
                {/if}
            </section>
        {:else if collections.length === 0}
            <EmptyList />
        {:else}
            <div class="collection-grid">
                {#each collections as collection (collection.id)}
                    <article class="collection-card"><button class="open-card" type="button" onclick={() => openCollection(collection.id)}>
                        <h3>{collection.title}</h3>
                        <p class="description">{collection.description}</p>
                        <p class="meta">Items: {collection.item_count}</p>
                        <div class="tags">
                            {#each collection.tags as tag (tag.id)}
                                <span class="tag">{tag.name}</span>
                            {/each}
                        </div>
                    </button>
                    <div class="card-actions"><button type="button" onclick={() => details = { collection, deleting: false }}>Edit details</button><button type="button" onclick={() => details = { collection, deleting: true }}>Delete</button></div>
                    </article>
                {/each}
            </div>
        {/if}
    </main>
    {#if details}<CollectionDetailsDialog collection={details.collection} deleting={details.deleting} onClose={() => details = null} onDone={() => {
        const deletingLast = details?.deleting && collections.length === 1 && hasPrev;
        details = null;
        if (deletingLast) void loadPrevByKeyset(); else void fetchCollections(currentFilters);
    }} />{/if}
</div>

<style>
    .open-card { text-align: left; width: 100%; display: grid; gap: .5rem; cursor: pointer; }
    .card-actions { display: flex; gap: .5rem; }
    .card-actions button { border: 1px solid #d1d5db; border-radius: 6px; padding: .3rem .6rem; cursor: pointer; }
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

    .collection-grid {
        padding: 1rem;
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
        gap: 0.8rem;
    }

    .collection-card {
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

    .collection-card h3 {
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

    .error-box {
        margin: 1rem;
        border: 1px solid #fecaca;
        background: #fef2f2;
        border-radius: 10px;
        padding: 0.8rem;
        color: #991b1b;
        display: flex;
        align-items: center;
        gap: 0.8rem;
    }

    .error-box p {
        margin: 0;
        font-weight: 600;
    }

    .login-link {
        border: 1px solid #991b1b;
        color: #991b1b;
        border-radius: 8px;
        padding: 0.25rem 0.6rem;
        text-decoration: none;
        font-weight: 600;
        background: #fff;
    }
</style>
