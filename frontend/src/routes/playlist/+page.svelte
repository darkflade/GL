<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { repositories } from "$lib/composition/repositories";
    import { getPlaylist } from "$lib/application/use-cases/get-playlist";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import PostCard from "$lib/features/feed/components/PostCard.svelte";
    import type { Playlist } from "$lib/domain/models/playlist";
    import type { UUID } from "$lib/domain";

    let loading = $state(false);
    let playlist = $state<Playlist | null>(null);

    $effect(() => {
        const id = page.url.searchParams.get("id") as UUID | null;
        if (!id) {
            goto("/playlists", {
                keepFocus: true,
                replaceState: true,
                noScroll: true,
            });
            return;
        }

        fetchPlaylist(id);
    });

    async function fetchPlaylist(id: UUID) {
        loading = true;
        try {
            playlist = await getPlaylist(repositories.playlists, id);
        } catch (error) {
            console.error(error);
            playlist = null;
        } finally {
            loading = false;
        }
    }
</script>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <Header />
    <header class="bg-white sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <a class="back-link" href="/playlists">Back to playlists</a>
        <h1 class="text-xl font-bold tracking-tight">Playlist</h1>
    </header>

    <main>
        {#if loading}
            <div class="flex items-center justify-center h-64">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
            </div>
        {:else if !playlist}
            <EmptyList />
        {:else}
            <section class="playlist-head">
                <h2>{playlist.title}</h2>
                {#if playlist.description}
                    <p class="description">{playlist.description}</p>
                {/if}
                <div class="tags">
                    {#each playlist.tags as tag (tag.id)}
                        <span class="tag">{tag.name}</span>
                    {/each}
                </div>
            </section>

            <section class="feed">
                {#each playlist.items as item (item.id)}
                    {#if "Post" in item.content}
                        <article class="feed-item">
                            <a href="/post?id={item.content.Post.id}" class="media-link">
                                <PostCard post={item.content.Post} size={"full"} />
                            </a>
                        </article>
                    {/if}
                {/each}
            </section>
        {/if}
    </main>
</div>

<style>
    .back-link {
        border: 1px solid #d1d5db;
        background: #fff;
        border-radius: 8px;
        padding: 0.35rem 0.7rem;
        text-decoration: none;
        color: #111827;
        margin-right: 0.75rem;
    }

    .playlist-head {
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .playlist-head h2 {
        margin: 0;
    }

    .description {
        margin: 0;
        color: #6b7280;
    }

    .feed {
        display: flex;
        flex-direction: column;
        gap: 2rem;
        padding: 1rem;
        align-items: center;

    }

    .feed-item {
        display: flex;
        min-height: calc(10vh - 9rem);
        background: #fff;
        border: 1px solid #e5e7eb;
        border-radius: 12px;
        overflow: hidden;
    }

    .media-link {
        display: block;
        width: 80rem;
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
</style>
