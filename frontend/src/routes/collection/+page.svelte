<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { repositories } from "$lib/composition/repositories";
    import { getCollection } from "$lib/application/use-cases/get-collection";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import { ApiError } from "$lib/infrastructure/http/client";
    import type { Collection } from "$lib/domain/models/collection";
    import type { UUID } from "$lib/domain";

    import CollectionEditor from "$lib/features/collections/components/CollectionEditor.svelte";

    import CollectionPlayback from "$lib/features/collections/components/CollectionPlayback.svelte";
    let presenting = $state<number | null>(null);
    let editing = $state(false);
    let loading = $state(false);
    let collection = $state<Collection | null>(null);
    let fetchError = $state("");
    let unauthorized = $state(false);

    $effect(() => {
        const id = page.url.searchParams.get("id") as UUID | null;
        if (!id) {
            goto("/collections", {
                keepFocus: true,
                replaceState: true,
                noScroll: true,
            });
            return;
        }

        fetchCollection(id);
    });

    async function fetchCollection(id: UUID) {
        presenting = null;
        editing = false;
        loading = true;
        fetchError = "";
        unauthorized = false;
        try {
            collection = await getCollection(repositories.collections, id);
        } catch (error) {
            console.error(error);
            collection = null;
            if (error instanceof ApiError && error.code === 401) {
                unauthorized = true;
                fetchError = "Session expired or missing. Please sign in.";
            } else if (error instanceof ApiError && error.code === 404) {
                fetchError = "Collection not found.";
            } else if (error instanceof Error) {
                fetchError = error.message;
            } else {
                fetchError = "Failed to load collection.";
            }
        } finally {
            loading = false;
        }
    }

    function getSignInHref(): string {
        const redirectTo = `${page.url.pathname}${page.url.search}`;
        return `/auth/login?redirect=${encodeURIComponent(redirectTo)}`;
    }
</script>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <Header />
    <header class="bg-white sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <a class="back-link" href="/collections">Back to collections</a>
        <h1 class="text-xl font-bold tracking-tight">Collection</h1>
    </header>

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
        {:else if !collection}
            <EmptyList />
        {:else}
            <section class="collection-head">
                <h2>{collection.title}</h2>
                {#if collection.description}
                    <p class="description">{collection.description}</p>
                {/if}
                <div class="tags">
                    {#each collection.tags as tag (tag.id)}
                        <span class="tag">{tag.name}</span>
                    {/each}
                </div>
            </section>

            {#if editing}
                <CollectionEditor {collection} onCancel={() => editing = false} onSaved={(updated) => { collection = updated; editing = false; }} />
            {:else}
            <div class="edit-actions"><button type="button" onclick={() => editing = true}>Edit items</button> <button type="button" disabled={!collection.items.length} onclick={() => presenting = -1}>Present collection</button></div>
            <section class="feed">
                {#each collection.items as item, index (item.id)}
                    {#if "Post" in item.content}
                        <article class="feed-item">
                            <button type="button" class="media-link" aria-label="Present {item.content.Post.title || 'post'}" onclick={() => presenting = index}>
                                {#if String(item.content.Post.file.media_type).toLowerCase() === 'picture'}
                                    <img src={`/api/files/full/${item.content.Post.file.id}`} alt={item.content.Post.title} loading="lazy" />
                                {:else if String(item.content.Post.file.media_type).toLowerCase() === 'video'}
                                    <video src={`/api/files/full/${item.content.Post.file.id}`} muted playsinline preload="metadata"></video>
                                    <span>▶ {item.content.Post.title || 'Video'}</span>
                                {:else}<span>♫ {item.content.Post.title || 'Audio'}</span>{/if}
                            </button>
                        </article>
                    {:else}
                        <article class="note-item"><p>{item.content.Note}</p></article>
                    {/if}
                {/each}
                {#if collection.items.length === 0}<p>No items yet. Use Edit items to add posts or notes.</p>{/if}
            </section>
            {/if}
        {/if}
    </main>
    {#if collection && presenting !== null}<CollectionPlayback {collection} start={presenting < 0 ? undefined : presenting} onClose={() => presenting = null} />{/if}
</div>

<style>
    .edit-actions { padding: 0 1rem; }
    .edit-actions button { background: #2563eb; color: white; border-radius: 8px; padding: .5rem 1rem; cursor: pointer; }
    .note-item { width: 100%; max-width: 80rem; padding: 1rem; background: white; border: 1px solid #e5e7eb; border-radius: 12px; white-space: pre-wrap; overflow-wrap: anywhere; }
    .feed-item { max-width: 100%; }
    .back-link {
        border: 1px solid #d1d5db;
        background: #fff;
        border-radius: 8px;
        padding: 0.35rem 0.7rem;
        text-decoration: none;
        color: #111827;
        margin-right: 0.75rem;
    }

    .collection-head {
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .collection-head h2 {
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

    .media-link img, .media-link video { width: 100%; max-height: 85vh; object-fit: contain; }
    .media-link span { display: block; padding: 1rem; }
    .media-link {
        cursor: pointer;
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
