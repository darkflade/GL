
<script lang="ts">
    import type { Post } from "$lib/domain/models/post";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { searchPosts } from "$lib/application/use-cases/search-posts";
    import { repositories } from "$lib/composition/repositories";
    import PostCard from "$lib/features/feed/components/PostCard.svelte";
    import PostSearchControls from "$lib/features/feed/components/PostSearchControls.svelte";
    import { buildSearchHref, queryFromUrl } from "$lib/features/feed/search-query";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import Header from "$lib/shared/components/layout/Header.svelte";
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
            console.error(e);
        }
    }

</script>

<Header/>

<div class="playlist-container">
    <div class="playlist-hero">
        <h1 class="page-title text-gradient">My Playlists</h1>
        <p class="page-subtitle">Your personal space for curated visual stories.</p>
    </div>

    <div class="filter-bar-sticky">
        <div class="filter-bar glass">
            <PostSearchControls value={textSearchValue} onQueryChange={handleSearchQuery} />
        </div>
    </div>

    <main class="playlist-content">
        {#if loading}
            <div class="loading-state">
                <div class="spinner"></div>
            </div>
        {:else if posts.length === 0}
            <EmptyList/>
        {:else}
            <div class="masonry-grid">
                {#each posts as post (post.id)}
                    <a href="/post?id={post.id}" class="post-link">
                        <PostCard {post} />
                    </a>
                {/each}
            </div>
        {/if}
    </main>
</div>

<style>
    .playlist-container {
        min-height: 100vh;
        padding-top: var(--spacing-2xl);
        padding-bottom: var(--spacing-2xl);
    }

    .playlist-hero {
        padding: var(--spacing-xl) var(--spacing-lg) var(--spacing-md);
        max-width: 800px;
        margin: 0 auto;
        text-align: center;
    }

    .page-title {
        font-size: clamp(2rem, 6vw, 3.5rem);
        font-weight: 800;
        line-height: 1.2;
        letter-spacing: -0.04em;
        margin-bottom: var(--spacing-md);
        position: relative;
        z-index: 1;
        padding: 0.1em 0;
    }

    .page-subtitle {
        font-size: var(--text-lg);
        color: var(--color-text-secondary);
        max-width: 600px;
        margin: 0 auto;
    }

    .filter-bar-sticky {
        position: sticky;
        top: calc(var(--header-height) + var(--spacing-md));
        z-index: 40;
        margin: 0 var(--spacing-lg) var(--spacing-2xl);
        display: flex;
        justify-content: center;
    }

    .filter-bar {
        width: 100%;
        max-width: 700px;
        padding: 8px;
        border-radius: var(--radius-full);
        box-shadow: var(--shadow-lg);
    }

    .playlist-content {
        padding: 0 var(--spacing-lg);
        max-width: 1800px;
        margin: 0 auto;
    }

    .masonry-grid {
        column-count: 1;
        column-gap: var(--spacing-lg);
    }

    .post-link {
        display: block;
        margin-bottom: var(--spacing-lg);
        text-decoration: none;
        color: inherit;
        break-inside: avoid;
        border-radius: var(--radius-lg);
        transition: transform 0.4s var(--ease-out);
    }

    .post-link:hover {
        transform: scale(0.99);
    }

    .loading-state {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 40vh;
    }

    .spinner {
        width: 48px;
        height: 48px;
        border: 2px solid var(--color-surface-hover);
        border-top-color: var(--color-accent);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    /* Responsive Columns */
    @media (min-width: 640px) {
        .masonry-grid {
            column-count: 2;
        }
    }

    @media (min-width: 1024px) {
        .masonry-grid {
            column-count: 3;
        }
    }

    @media (min-width: 1280px) {
        .masonry-grid {
            column-count: 4;
        }
    }
    
    @media (min-width: 1600px) {
        .masonry-grid {
            column-count: 5;
        }
    }
</style>
