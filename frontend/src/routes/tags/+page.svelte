<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import EmptyList from "$lib/shared/components/layout/EmptyList.svelte";
    import { repositories } from "$lib/composition/repositories";
    import { listTagRelations, listTags } from "$lib/application/use-cases/list-tags";
    import type { Tag, TagRelation } from "$lib/domain/models/tag";
    import { TagCategory } from "$lib/domain/models/tag";
    import type { KeysetCursor } from "$lib/domain/value-objects/search";

    type TabKey = "tags" | "tag_relations";

    let activeTab = $state<TabKey>("tags");
    let loadError = $state("");

    let tags = $state<Tag[]>([]);
    let tagsInitialized = $state(false);
    let tagsLoading = $state(false);
    let tagsHasNext = $state(false);
    let tagsNextCursor = $state<KeysetCursor | null>(null);

    let relations = $state<TagRelation[]>([]);
    let relationsInitialized = $state(false);
    let relationsLoading = $state(false);
    let relationsHasNext = $state(false);
    let relationsNextCursor = $state<KeysetCursor | null>(null);

    let sentinel = $state<HTMLDivElement | null>(null);

    function normalizeTab(value: string | null): TabKey {
        return value === "tag_relations" ? "tag_relations" : "tags";
    }

    function formatCategory(category: string | number): string {
        if (typeof category === "number") {
            return TagCategory[category] ?? String(category);
        }
        return category;
    }

    $effect(() => {
        const nextTab = normalizeTab(page.url.searchParams.get("tab"));
        if (activeTab !== nextTab) {
            activeTab = nextTab;
        }

        if (activeTab === "tags" && !tagsInitialized && !tagsLoading) {
            void loadTagsPage();
        }
        if (activeTab === "tag_relations" && !relationsInitialized && !relationsLoading) {
            void loadRelationsPage();
        }
    });

    $effect(() => {
        if (!sentinel) return;

        const observer = new IntersectionObserver(
            (entries) => {
                if (!entries.some((entry) => entry.isIntersecting)) return;
                void loadMore();
            },
            { rootMargin: "220px 0px" }
        );

        observer.observe(sentinel);
        return () => observer.disconnect();
    });

    async function changeTab(tab: TabKey) {
        if (activeTab === tab) return;
        await goto(`/tags?tab=${tab}`, {
            keepFocus: true,
            replaceState: false,
            noScroll: true,
        });
    }

    async function refreshActiveTab() {
        loadError = "";
        if (activeTab === "tags") {
            tags = [];
            tagsNextCursor = null;
            tagsHasNext = false;
            tagsInitialized = false;
            await loadTagsPage();
            return;
        }

        relations = [];
        relationsNextCursor = null;
        relationsHasNext = false;
        relationsInitialized = false;
        await loadRelationsPage();
    }

    async function loadMore() {
        if (activeTab === "tags") {
            if (!tagsInitialized || tagsLoading || !tagsHasNext || !tagsNextCursor) return;
            await loadTagsPage(tagsNextCursor);
            return;
        }

        if (!relationsInitialized || relationsLoading || !relationsHasNext || !relationsNextCursor) return;
        await loadRelationsPage(relationsNextCursor);
    }

    async function loadTagsPage(cursor?: KeysetCursor) {
        if (tagsLoading) return;
        tagsLoading = true;
        loadError = "";
        try {
            const response = await listTags(repositories.tags, cursor);
            tags = cursor ? [...tags, ...response.tags] : response.tags;
            tagsHasNext = response.has_next;
            tagsNextCursor = response.next_cursor ?? null;
            tagsInitialized = true;
        } catch (error) {
            console.error(error);
            loadError = "Failed to load tags.";
            if (!cursor) tags = [];
        } finally {
            tagsLoading = false;
        }
    }

    async function loadRelationsPage(cursor?: KeysetCursor) {
        if (relationsLoading) return;
        relationsLoading = true;
        loadError = "";
        try {
            const response = await listTagRelations(repositories.tags, cursor);
            relations = cursor ? [...relations, ...response.relations] : response.relations;
            relationsHasNext = response.has_next;
            relationsNextCursor = response.next_cursor ?? null;
            relationsInitialized = true;
        } catch (error) {
            console.error(error);
            loadError = "Failed to load tag relations.";
            if (!cursor) relations = [];
        } finally {
            relationsLoading = false;
        }
    }
</script>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <Header />
    <header class="bg-white sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <h1 class="text-xl font-bold tracking-tight">Tags</h1>
    </header>

    <div class="tabs">
        <button class:active={activeTab === "tags"} type="button" onclick={() => changeTab("tags")}>
            tags
        </button>
        <button
            class:active={activeTab === "tag_relations"}
            type="button"
            onclick={() => changeTab("tag_relations")}
        >
            tag_relations
        </button>
        <button class="refresh-btn" type="button" onclick={refreshActiveTab}>
            Refresh
        </button>
    </div>

    <main class="content">
        {#if loadError}
            <p class="error">{loadError}</p>
        {/if}

        {#if activeTab === "tags"}
            {#if !tagsInitialized && tagsLoading}
                <p class="status">Loading tags...</p>
            {:else if tags.length === 0}
                <EmptyList />
            {:else}
                <section class="list">
                    {#each tags as tag (tag.id)}
                        <article class="card">
                            <div class="card-head">
                                <h3>{tag.name}</h3>
                                <span class="count">{tag.count}</span>
                            </div>
                            <p class="meta">Category: {formatCategory(tag.category)}</p>
                        </article>
                    {/each}
                </section>
            {/if}
        {:else}
            {#if !relationsInitialized && relationsLoading}
                <p class="status">Loading tag relations...</p>
            {:else if relations.length === 0}
                <EmptyList />
            {:else}
                <section class="list">
                    {#each relations as relation (relation.id)}
                        <article class="card">
                            <div class="relation-row">
                                <span>{relation.parent_name}</span>
                                <span class="arrow">→</span>
                                <span>{relation.child_name}</span>
                            </div>
                            <p class="meta">
                                Parent: {relation.parent_count} | Child: {relation.child_count} | Score: {relation.score}
                            </p>
                        </article>
                    {/each}
                </section>
            {/if}
        {/if}

        <div class="status-area">
            {#if activeTab === "tags" && tagsInitialized && tagsLoading}
                <p class="status">Loading more tags...</p>
            {/if}
            {#if activeTab === "tag_relations" && relationsInitialized && relationsLoading}
                <p class="status">Loading more relations...</p>
            {/if}
            {#if activeTab === "tags" && tagsInitialized && !tagsHasNext && tags.length > 0}
                <p class="status">All tags loaded.</p>
            {/if}
            {#if activeTab === "tag_relations" && relationsInitialized && !relationsHasNext && relations.length > 0}
                <p class="status">All relations loaded.</p>
            {/if}
        </div>

        <div class="sentinel" bind:this={sentinel}></div>
    </main>
</div>

<style>
    .tabs {
        display: flex;
        align-items: center;
        gap: 0.55rem;
        padding: 0.75rem 1rem;
        background: #f9fafb;
        border-bottom: 1px solid #e5e7eb;
    }

    .tabs button {
        border: 1px solid #d1d5db;
        background: #fff;
        border-radius: 8px;
        padding: 0.35rem 0.75rem;
        cursor: pointer;
        color: #1f2937;
    }

    .tabs button.active {
        background: #111827;
        border-color: #111827;
        color: #fff;
    }

    .refresh-btn {
        margin-left: auto;
    }

    .content {
        padding: 1rem;
    }

    .list {
        display: grid;
        gap: 0.7rem;
    }

    .card {
        border: 1px solid #e5e7eb;
        border-radius: 10px;
        background: #fff;
        padding: 0.75rem 0.85rem;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .card-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.75rem;
    }

    .card h3 {
        margin: 0;
        font-size: 1rem;
    }

    .count {
        font-variant-numeric: tabular-nums;
        color: #374151;
        font-weight: 600;
    }

    .relation-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-weight: 600;
    }

    .arrow {
        color: #6b7280;
    }

    .meta {
        margin: 0;
        color: #6b7280;
        font-size: 0.88rem;
    }

    .error {
        color: #b91c1c;
        font-weight: 600;
        margin: 0 0 0.75rem 0;
    }

    .status-area {
        min-height: 1.5rem;
    }

    .status {
        margin: 0.75rem 0 0;
        color: #4b5563;
    }

    .sentinel {
        height: 1px;
    }
</style>
