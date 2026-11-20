<script lang="ts">
    import { repositories } from "$lib/composition/repositories";
    import { searchCollections } from "$lib/application/use-cases/search-collections";
    import { updateCollection } from "$lib/application/use-cases/update-collection";
    import { createCollection } from "$lib/application/use-cases/create-collection";
    import { getPost } from "$lib/application/use-cases/get-post";
    import type { KeysetCursorDto } from "$lib/infrastructure/repositories/dto";
    import type { CollectionSummary } from "$lib/domain/models/collection";
    import type { SearchPostsQuery, UUID } from "$lib/domain";

    let {
        open = false,
        postIds = [] as UUID[],
        onClose,
        onDone,
    } = $props<{
        open?: boolean;
        postIds: UUID[];
        onClose: () => void;
        onDone?: () => void;
    }>();

    const SEARCH_LIMIT = 100;

    let loadingCollections = $state(false);
    let saving = $state(false);
    let errorText = $state("");
    let collections = $state<CollectionSummary[]>([]);
    let targetMode = $state<"existing" | "new">("existing");
    let selectedCollectionId = $state<UUID | "">("");
    let newTitle = $state("");
    let newDescription = $state("");

    $effect(() => {
        if (!open) return;
        errorText = "";
        targetMode = "existing";
        selectedCollectionId = "";
        void loadCollections();
    });

    function uniquePostIds(): UUID[] {
        const ids = postIds as UUID[];
        return [...new Set(ids)];
    }

    function buildSearchQuery(cursor?: KeysetCursorDto): SearchPostsQuery {
        return {
            tag_query: {
                must: [],
                should: [],
                must_not: [],
            },
            text_query: "",
            cursor: cursor
                ? {
                      mode: "keyset",
                      direction: cursor.direction,
                      last_id: cursor.last_id,
                      last_score: cursor.last_score,
                      limit: cursor.limit,
                  }
                : {
                      mode: "keyset",
                      limit: SEARCH_LIMIT,
                  },
        };
    }

    async function loadCollections() {
        loadingCollections = true;
        errorText = "";
        try {
            const all: CollectionSummary[] = [];
            let nextCursor: KeysetCursorDto | undefined;
            let pageGuard = 20;
            while (pageGuard > 0) {
                const response = await searchCollections(repositories.collections, buildSearchQuery(nextCursor));
                all.push(...response.collections);
                if (!response.has_next || !response.next_cursor) {
                    break;
                }
                nextCursor = response.next_cursor;
                pageGuard -= 1;
            }

            const byId = new Map<UUID, CollectionSummary>();
            for (const item of all) {
                byId.set(item.id, item);
            }

            collections = [...byId.values()];
        } catch (error) {
            console.error(error);
            errorText = "Failed to load collections";
            collections = [];
        } finally {
            loadingCollections = false;
        }
    }

    async function mergeTagIds(ids: UUID[]): Promise<UUID[]> {
        const fetchedPosts = await Promise.all(ids.map((id) => getPost(repositories.posts, id).catch(() => null)));
        const tagSet = new Set<UUID>();
        for (const post of fetchedPosts) {
            if (!post) continue;
            for (const tag of post.tags) {
                tagSet.add(tag.id);
            }
        }
        return [...tagSet];
    }

    async function applyToExistingCollection(collectionId: UUID, ids: UUID[]) {
        const itemEvents = [...ids].reverse().map((postId) => ({
            op: "add" as const,
            after_id: null,
            content: {
                type: "post" as const,
                post_id: postId,
            },
        }));

        await updateCollection(repositories.collections, collectionId, {
            item_events: itemEvents,
        });
    }

    async function createNewCollection(ids: UUID[]) {
        const title = newTitle.trim();
        const tagIds = await mergeTagIds(ids);
        const items = ids.map((postId, index) => ({
            position: index + 1,
            content: {
                type: "post" as const,
                post_id: postId,
            },
        }));

        await createCollection(repositories.collections, {
            title,
            description: newDescription.trim() ? newDescription.trim() : null,
            tag_ids: tagIds.length > 0 ? tagIds : null,
            items,
        });
    }

    async function submit() {
        const ids = uniquePostIds();
        if (ids.length === 0) {
            errorText = "No posts selected";
            return;
        }

        saving = true;
        errorText = "";
        try {
            if (targetMode === "existing") {
                if (!selectedCollectionId) {
                    errorText = "Select a collection";
                    return;
                }
                await applyToExistingCollection(selectedCollectionId, ids);
            } else {
                if (!newTitle.trim()) {
                    errorText = "Title is required";
                    return;
                }
                await createNewCollection(ids);
            }

            onDone?.();
            onClose();
        } catch (error) {
            console.error(error);
            errorText = "Failed to apply collection changes";
        } finally {
            saving = false;
        }
    }
</script>

{#if open}
    <button class="backdrop" type="button" aria-label="Close dialog" onclick={onClose}></button>
    <div class="dialog" role="dialog" aria-modal="true" aria-label="Add to collection">
        <div class="head">
            <h3>Add To Collection</h3>
            <button type="button" class="close-btn" onclick={onClose}>Close</button>
        </div>

        <p class="meta">Selected posts: {uniquePostIds().length}</p>

        <div class="mode-row">
            <button
                type="button"
                class:active={targetMode === "existing"}
                class="mode-btn"
                onclick={() => (targetMode = "existing")}
            >
                Existing
            </button>
            <button
                type="button"
                class:active={targetMode === "new"}
                class="mode-btn"
                onclick={() => (targetMode = "new")}
            >
                Add To New
            </button>
        </div>

        {#if targetMode === "existing"}
            {#if loadingCollections}
                <p class="meta">Loading collections...</p>
            {:else if collections.length === 0}
                <p class="meta">No collections found</p>
            {:else}
                <div class="collections-list">
                    {#each collections as collection (collection.id)}
                        <button
                            type="button"
                            class:active={selectedCollectionId === collection.id}
                            class="collection-item"
                            onclick={() => (selectedCollectionId = collection.id)}
                        >
                            <span class="title">{collection.title}</span>
                            <span class="count">{collection.item_count} items</span>
                        </button>
                    {/each}
                </div>
            {/if}
        {:else}
            <div class="field">
                <label for="collection-title">Title</label>
                <input id="collection-title" class="input" bind:value={newTitle} />
            </div>
            <div class="field">
                <label for="collection-description">Description</label>
                <textarea id="collection-description" class="input" rows="3" bind:value={newDescription}></textarea>
            </div>
        {/if}

        {#if errorText}
            <p class="error">{errorText}</p>
        {/if}

        <div class="actions">
            <button type="button" class="btn" onclick={onClose} disabled={saving}>Cancel</button>
            <button type="button" class="btn primary" onclick={submit} disabled={saving}>
                {saving ? "Saving..." : "Confirm"}
            </button>
        </div>
    </div>
{/if}

<style>
    .backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        border: 0;
        z-index: 60;
    }

    .dialog {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: min(520px, calc(100vw - 2rem));
        max-height: calc(100vh - 2rem);
        overflow: auto;
        border: 1px solid #e5e7eb;
        border-radius: 12px;
        background: #fff;
        padding: 1rem;
        z-index: 70;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .head {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .head h3 {
        margin: 0;
    }

    .meta {
        margin: 0;
        color: #6b7280;
    }

    .close-btn,
    .btn,
    .mode-btn,
    .collection-item {
        border: 1px solid #d1d5db;
        background: #fff;
        border-radius: 8px;
        padding: 0.4rem 0.7rem;
        cursor: pointer;
    }

    .mode-row {
        display: flex;
        gap: 0.5rem;
    }

    .mode-btn.active {
        border-color: #2563eb;
        color: #2563eb;
    }

    .collections-list {
        display: flex;
        flex-direction: column;
        gap: 0.45rem;
        max-height: 280px;
        overflow: auto;
    }

    .collection-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        text-align: left;
    }

    .collection-item.active {
        border-color: #2563eb;
        background: #eff6ff;
    }

    .collection-item .title {
        font-weight: 600;
    }

    .collection-item .count {
        color: #6b7280;
        font-size: 0.85rem;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .input {
        border: 1px solid #d1d5db;
        border-radius: 8px;
        padding: 0.45rem 0.6rem;
    }

    .error {
        margin: 0;
        color: #b91c1c;
    }

    .actions {
        display: flex;
        gap: 0.5rem;
        justify-content: flex-end;
    }

    .btn.primary {
        border-color: #2563eb;
        background: #2563eb;
        color: #fff;
    }

    .btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
