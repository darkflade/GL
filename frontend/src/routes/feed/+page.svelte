
<script lang="ts">
    import type {Post, Tag} from "$lib/domain/models";
    import {onMount} from "svelte";
    import {searchAllPosts, searchPosts} from "$lib/features/feed/api";
    import PostCard from "$lib/features/feed/PostCard.svelte";
    import {api} from "$lib/api/client";
    import TagSearch from "$lib/features/feed/TagSearch.svelte";


    let posts = $state<Post[]>([])
    let loading = $state(false)

    onMount(async () => {
        try {
            console.log("Loading posts");
            let timeout = setTimeout(async () => {
                posts = await searchAllPosts()
                console.log("Timeout Called")
            }, 100)

        } catch (e) {
            console.error(e)
        } finally {
            loading = false
        }
    })

    async function handleTagsChange(tags: Tag[]) {
        const mustIds = tags.map(t => t.id);


        loading = true;
        try {
            console.log("Searching posts in Parent with IDs:", mustIds);
            posts = await searchPosts({
                must: mustIds,
                should: [],
                must_not: []
            });
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }
</script>


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
            <div class="text-center py-20 text-gray-500">
                Nothing was discovered
            </div>
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
