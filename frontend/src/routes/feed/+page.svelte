
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

    $effect(() => {
        const filters = queryFromUrl(page.url.searchParams)
        textSearchValue = toSearchInput(filters)
        fetchData(filters)
    })

    async function fetchData(filters: SearchPostsQuery) {
        loading = true;
        try {
            posts = await searchPosts(repositories.posts, filters);
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

</script>

<Header/>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <header class="bg-white border-b sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <h1 class="text-xl font-bold tracking-tight">
            Glab Storage
        </h1>
        <PostSearchControls value={textSearchValue} onQueryChange={handleSearchQuery} />
    </header>
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
                    <PostCard {post} />
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
        padding: 1rem;
    }
</style>
