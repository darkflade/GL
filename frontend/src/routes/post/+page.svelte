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

{#if loading}

{:else if post}
    <Header/>
    <header class="header">
        <div class="name">
            {`${post.title} ||| ${post.id}`}
        </div>
        <div class="description">
            Description: {post.description}
        </div>
    </header>
    <main class="main">

        <div class="tags">
            <p>Tags</p>
            {#each post.tags as tag}
                <div class="tag">
                    {tag.value}
                </div>
            {/each}
        </div>
        <div class="content">
            <PostCard post={post} size="full"/>
        </div>

        <div class="post_content">

        </div>
    </main>
{:else}
    <EmptyList/>
{/if}

<style>
    .header {
        display: flex;
        flex-direction: column;
        gap: 8px;
        margin-top: 1rem;

    }

    .name {
        font-size: 20px;
        font-weight: 600;
    }

    .description {
        margin-bottom: 10px;
        color: #666;
    }

    .tags {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 6px;
        margin-bottom: 12px;
    }

    .tags p {
        margin: 0;
        font-weight: 600;
    }

    .tag {
        padding: 2px 8px;
        border: 1px solid #000000;
        border-radius: 999px;
        font-size: 12px;
    }

    .content {
        margin-bottom: 16px;
    }

</style>
