
<script lang="ts">
    import type { Post } from "$lib/domain/models/post";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { searchPosts } from "$lib/application/use-cases/search-posts";
    import { repositories } from "$lib/composition/repositories";
    import PostCard from "$lib/features/feed/components/PostCard.svelte";
    import PostSearchControls from "$lib/features/feed/components/PostSearchControls.svelte";
    import { buildSearchHref, queryFromUrl } from "$lib/features/feed/search-query";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import { toSearchInput } from "$lib/utils/search";
    import type { SearchPostsQuery } from "$lib/domain";

    let posts = $state<Post[]>([])
    let loading = $state(false)
    let textSearchValue = $state("")
    let currentPage = $state(0)
    let currentFilters = $state<SearchPostsQuery>({
        tag_query: {
            must: [],
            should: [],
            must_not: [],
        },
        cursor: { page: 0 }
    })

    $effect(() => {
        const filters = queryFromUrl(page.url.searchParams)
        currentFilters = filters
        currentPage = filters.cursor.page
        textSearchValue = toSearchInput(filters)
        fetchData(filters)
    })

    async function fetchData(filters: SearchPostsQuery) {
        loading = true;
        try {
            let ServerResponse = await searchPosts(repositories.posts, filters)
            posts = ServerResponse.posts

        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }


    async function handleSearchQuery(query: SearchPostsQuery) {
        try {
            const newLink = buildSearchHref(page.url.pathname, query);
            goto(newLink, {
                keepFocus: true,
                replaceState: false,
                noScroll: true
            })

        } catch (e) {
            console.error(e)
        }
    }

    async function changePage(nextPage: number) {
        const query: SearchPostsQuery = {
            ...currentFilters,
            cursor: { page: Math.max(0, nextPage) }
        };
        await handleSearchQuery(query);
    }

</script>

<div class="min-h-screen bg-gray-50 text-gray-900">
        <Header/>
    <header class="bg-white sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <h1 class="text-xl font-bold tracking-tight">
            Glab Storage
        </h1>
        <PostSearchControls value={textSearchValue} onQueryChange={handleSearchQuery} />
    </header>
    <div class="pager">
        <button class="pager-btn" type="button" onclick={() => changePage(currentPage - 1)} disabled={currentPage <= 0}>
            Prev
        </button>
        <span class="pager-label">Page {currentPage + 1}</span>
        <button class="pager-btn" type="button" onclick={() => changePage(currentPage + 1)}>
            Next
        </button>
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
