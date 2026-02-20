<script lang="ts">
    import type { Post } from "$lib/domain/models/post";
    import {page} from "$app/state";
    import {repositories} from "$lib/composition/repositories";
    import {postIDFromURL} from "$lib/features/post/utils";
    import type {UUID} from "$lib/domain";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import {getPost} from "$lib/application/use-cases/get-post";
    import PostCard from "$lib/features/feed/components/PostCard.svelte";
    import Header from "$lib/shared/components/layout/Header.svelte";

    let loading = $state(false)
    let post = $state<Post>()
    let empty = $state(true)

    $effect(() => {
        const id = postIDFromURL(page.url.searchParams)
        if (id != null) {
            fetchData(id)
        } else {
            empty = true
        }

    })

    async function fetchData(id: UUID) {
        loading = true;
        try {
            post = await getPost(repositories.posts, id);
            empty = !post
        } catch (e) {
            console.error(e);
            empty = true
        } finally {
            loading = false
        }
    }
</script>

<Header/>

<div class="post-page">
    {#if loading}
        <div class="loading-state">
            <div class="spinner"></div>
        </div>
    {:else if post}
        <main class="post-container">
            <div class="post-header">
                <h1 class="post-title text-gradient">{post.title}</h1>
                <p class="post-id">ID: {post.id}</p>
                
                <div class="post-tags">
                    {#each post.tags as tag}
                        <span class="glass-tag">{tag.value}</span>
                    {/each}
                </div>
                
                <p class="post-description">{post.description}</p>
            </div>

            <div class="post-media-content">
                <PostCard post={post} size="full"/>
            </div>
        </main>
    {:else}
        <EmptyList/>
    {/if}
</div>

<style>
    .post-page {
        min-height: calc(100vh - var(--header-height));
        padding-top: var(--spacing-xl);
        padding-bottom: var(--spacing-2xl);
    }

    .post-container {
        max-width: 1000px;
        margin: 0 auto;
        padding: 0 var(--spacing-lg);
    }

    .post-header {
        margin-bottom: var(--spacing-2xl);
    }

    .post-title {
        font-size: clamp(2.5rem, 5vw, 4rem);
        font-weight: 800;
        letter-spacing: -0.04em;
        margin-bottom: var(--spacing-xs);
        line-height: 1.1;
    }

    .post-id {
        font-size: var(--text-xs);
        color: var(--color-text-muted);
        font-family: monospace;
        letter-spacing: 0.05em;
        margin-bottom: var(--spacing-md);
    }

    .post-tags {
        display: flex;
        flex-wrap: wrap;
        gap: var(--spacing-sm);
        margin-bottom: var(--spacing-lg);
    }

    .glass-tag {
        padding: 6px 14px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: var(--radius-full);
        font-size: var(--text-xs);
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--color-accent-vibrant);
        backdrop-filter: blur(4px);
    }

    .post-description {
        font-size: var(--text-lg);
        color: var(--color-text-secondary);
        line-height: 1.6;
        max-width: 800px;
    }

    .post-media-content {
        border-radius: var(--radius-xl);
        overflow: hidden;
        box-shadow: var(--shadow-xl);
        border: 1px solid var(--color-border-bright);
    }

    .loading-state {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 50vh;
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
</style>
