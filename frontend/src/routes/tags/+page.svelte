<script lang="ts">
    import { page } from '$app/state';
    import { goto } from '$app/navigation';
    import Header from '$lib/shared/components/layout/Header.svelte';
    import { repositories } from '$lib/composition/repositories';
    import type { Tag } from '$lib/domain/models/tag';
    import type { KeysetCursor } from '$lib/domain/value-objects/search';
    import { tagManagement, categoryName, type Neighborhood } from '$lib/features/tags/management';
    import TagEditor from '$lib/features/tags/components/TagEditor.svelte';
    import TagFinder from '$lib/features/tags/components/TagFinder.svelte';
    import TagGraph from '$lib/features/tags/components/TagGraph.svelte';
    let tab = $state<'tags' | 'graph' | 'aliases'>('tags');
    let tags = $state<Tag[]>([]), details = $state<Neighborhood | null>(null);
    let query = $state(''), activeQuery = $state('');
    let loading = $state(false), detailLoading = $state(false), error = $state(''), busy = $state(false);
    let cursor = $state<KeysetCursor | undefined>();
    let initialized = false; let listVersion = 0, detailVersion = 0;
    let graphDirty = $state(false);
    let aliasTarget = $state<Tag | null>(null);
    $effect(() => {
        const value = page.url.searchParams.get('tab');
        tab = value === 'tag_relations' || value === 'graph' ? 'graph' : value === 'aliases' ? 'aliases' : 'tags';
        if (!initialized) { initialized = true; void loadList(); }
    });
    async function changeTab(value: typeof tab) {
        if (tab === value) return;
        if (graphDirty && !window.confirm('Discard unsaved graph changes?')) return;
        graphDirty = false; await goto(`/tags?tab=${value}`, { noScroll: true });
    }
    async function loadList(more = false) {
        const request = ++listVersion; loading = true; error = '';
        if (!more) activeQuery = query.trim();
        try {
            if (activeQuery) {
                const result = await repositories.tags.searchTags(activeQuery);
                if (request !== listVersion) return;
                tags = result; cursor = undefined;
            } else {
                const result = await repositories.tags.listTags(more ? cursor : undefined);
                if (request !== listVersion) return;
                tags = more ? [...tags, ...result.tags] : result.tags;
                cursor = result.has_next ? result.next_cursor ?? undefined : undefined;
            }
        } catch { if (request === listVersion) error = 'Could not load tags. Try again.'; }
        finally { if (request === listVersion) loading = false; }
    }
    async function selectTag(id: string, force = false) {
        if (!force && graphDirty && !window.confirm('Discard unsaved graph changes?')) return;
        const request = ++detailVersion; graphDirty = false; detailLoading = true; error = ''; aliasTarget = null;
        try { const result = await tagManagement.get(id); if (request === detailVersion) details = result; }
        catch { if (request === detailVersion) error = 'Could not load tag details. It may have been deleted.'; }
        finally { if (request === detailVersion) detailLoading = false; }
    }
    async function changed(id: string | null) {
        if (id) await selectTag(id, true); else { ++detailVersion; details = null; }
        await loadList();
    }
    async function aliasAction(id: string, remove = false) {
        if (!details) return; busy = true; error = '';
        try { await tagManagement.relations([{ op: remove ? 'unalias' : 'alias', tag_id: details.tag.id, alias_ids: [id] }]); await selectTag(details.tag.id, true); }
        catch { error = 'Could not update aliases. Try again.'; }
        finally { busy = false; }
    }
</script>
<div class="page">
    <Header />
    <header class="page-head"><div><h1>Tags</h1><p>Manage names, hierarchy and aliases</p></div><nav><button class:active={tab === 'tags'} onclick={() => changeTab('tags')}>Tag editor</button><button class:active={tab === 'graph'} onclick={() => changeTab('graph')}>Hierarchy graph</button><button class:active={tab === 'aliases'} onclick={() => changeTab('aliases')}>Aliases ↔</button></nav></header>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <main>
        <aside>
            <form onsubmit={(event) => { event.preventDefault(); void loadList(); }}><label for="tag-search">Find a tag</label><div class="search"><input id="tag-search" bind:value={query} placeholder="Name or UUID" /><button disabled={loading}>Search</button></div></form>
            <div class="tag-list" aria-label="Tags">{#each tags as tag (tag.id)}<button class:selected={details?.tag.id === tag.id} onclick={() => selectTag(tag.id)}><span><strong>{tag.name}</strong><small>{tag.count} posts</small></span><small>{categoryName(tag.category)}</small></button>{/each}</div>
            {#if loading}<p role="status">Loading tags…</p>{:else if !tags.length}<p>No tags found.</p>{/if}
            {#if cursor}<button disabled={loading} onclick={() => loadList(true)}>Load more</button>{/if}
            {#if activeQuery && tags.length === 50}<small>First 50 results. Narrow your search.</small>{/if}
        </aside>
        <section class="workspace">
            {#if detailLoading}<p role="status">Loading tag details…</p>
            {:else if !details}<div class="empty"><h2>Select a tag</h2><p>Search by name or UUID, then select a tag to edit it or explore its branches.</p></div>
            {:else}
                {#key details.tag.id}
                    {#if tab === 'tags'}<TagEditor {details} onChanged={changed} />
                    {:else if tab === 'graph'}<TagGraph {details} onSaved={() => changed(details!.tag.id)} onFocus={(id) => selectTag(id)} onDirty={(dirty) => graphDirty = dirty} />
                    {:else}
                        <section class="aliases"><h2>Aliases · {details.tag.name}</h2><code>{details.tag.id}</code><p>Aliases are bidirectional synonyms, separate from parent → child branches. They keep both tags; use Merge to combine and delete a duplicate.</p>
                            <fieldset disabled={busy}>
                                {#each details.aliases as alias (alias.id)}<div class="alias"><div><strong>{alias.name}</strong><code>{alias.id}</code><small>{categoryName(alias.category)} · {alias.count} posts</small></div><button onclick={() => aliasAction(alias.id, true)}>Remove alias</button></div>{/each}
                                {#if !details.aliases.length}<p>No aliases yet.</p>{/if}
                                <TagFinder label="Find alias to add" exclude={[details.tag.id, ...details.aliases.map(tag => tag.id)]} onSelect={(tag) => aliasTarget = tag} />
                                {#if aliasTarget}<p>Connect {details.tag.name} ↔ {aliasTarget.name}</p><button onclick={() => aliasAction(aliasTarget!.id)}>Add bidirectional alias</button>{/if}
                            </fieldset>
                        </section>
                    {/if}
                {/key}
            {/if}
        </section>
    </main>
</div>
<style>
    .page { min-height: 100vh; background: #f8fafc; color: #0f172a; } .page-head { padding: 1.25rem 1.5rem; display: flex; justify-content: space-between; align-items: center; gap: 1rem; background: white; border-bottom: 1px solid #e2e8f0; } h1 { font-size: 1.5rem; font-weight: 700; } h2 { font-size: 1.25rem; font-weight: 600; } p,small { color: #64748b; } nav,.search { display: flex; gap: .5rem; flex-wrap: wrap; }
    button,input { border: 1px solid #cbd5e1; background: white; border-radius: 8px; padding: .5rem .7rem; } button { cursor: pointer; } button:disabled { opacity: .5; cursor: default; } .active { color: #1d4ed8; background: #eff6ff; border-color: #93c5fd; } input { flex: 1; min-width: 0; width: 100%; } label { display: block; margin-bottom: .4rem; font-weight: 500; }
    main { display: grid; grid-template-columns: 300px minmax(0,1fr); gap: 1.25rem; padding: 1.25rem; } aside,.workspace { border: 1px solid #e2e8f0; border-radius: 12px; background: white; padding: 1rem; min-width: 0; } aside { display: flex; flex-direction: column; gap: .8rem; align-self: start; } .tag-list { display: grid; gap: .35rem; max-height: 65vh; overflow: auto; } .tag-list button { display: grid; gap: .25rem; text-align: left; border-color: transparent; } .tag-list button:hover { background: #f1f5f9; } .tag-list span { display: flex; gap: .4rem; justify-content: space-between; } .tag-list strong { overflow-wrap: anywhere; } .selected { background: #eff6ff !important; border-color: #93c5fd !important; } .empty { padding: 4rem 1rem; text-align: center; } .error { color: #b91c1c; padding: .7rem 1.5rem; }
    .aliases,fieldset { display: grid; gap: 1rem; } fieldset { padding: 0; border: 0; } .alias { display: flex; justify-content: space-between; align-items: center; gap: 1rem; padding: .8rem; background: #f8fafc; border-radius: 8px; } .alias div { display: grid; gap: .25rem; } code { font-size: .75rem; color: #64748b; overflow-wrap: anywhere; }
    @media(max-width:900px) { main { grid-template-columns: 1fr; } .page-head { flex-direction: column; align-items: start; } .tag-list { max-height: 240px; } }
</style>
