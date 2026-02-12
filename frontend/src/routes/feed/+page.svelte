
<script lang="ts">
    import type { Post } from "$lib/domain/models/post";
    import type { Tag } from "$lib/domain/models/tag";
    import { page } from '$app/state';
    import { goto } from '$app/navigation'
    import { searchPosts } from "$lib/application/use-cases/search-posts";
    import { repositories } from "$lib/composition/repositories";
    import PostCard from "$lib/features/feed/components/PostCard.svelte";
    import TagSearch from "$lib/shared/components/layout/TagSearch.svelte";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import {deserializeQuery, serializeQuery} from "$lib";
    import type {SearchPostsQuery} from "$lib/domain";


    let posts = $state<Post[]>([])
    let loading = $state(false)

    $effect(() => {
        const filters = deserializeQuery(page.url.searchParams)
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


    async function handleTagsChange(tags: Tag[]) {
        const mustIds = tags.map(t => t.id);


        loading = true;
        try {
            console.log("Searching posts in Parent with IDs:", mustIds);
            let queryString = serializeQuery({
                must: mustIds,
                should: [],
                must_not: []
            });

            const newLink = queryString ? `?${queryString}` : page.url.pathname;

            goto(newLink, {
                keepFocus: true,
                replaceState: false,
                noScroll: true
            })

        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }

</script>

<Header/>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <header class="bg-white border-b sticky top-0 z-20 px-6 py-3 flex items-center justify-between shadow-sm">
        <h1 class="text-xl font-bold tracking-tight">
            Glab Storage
        </h1>
        <TagSearch onChange={handleTagsChange}/>
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
