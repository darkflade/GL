<script lang="ts">
    import { repositories } from '$lib/composition/repositories';
    import type { Tag } from '$lib/domain/models/tag';
    import { categoryName } from '../management';
    let { onSelect, label = 'Find tag', exclude = [] } = $props<{ onSelect: (tag: Tag) => void; label?: string; exclude?: string[] }>();
    let query = $state(''); let results = $state<Tag[]>([]); let loading = $state(false); let error = $state(''); let version = 0;
    async function search() {
        const request = ++version;
        if (!query.trim()) { results = []; return; }
        loading = true; error = '';
        try { const tags = await repositories.tags.searchTags(query.trim()); if (request === version) results = tags; }
        catch { if (request === version) error = 'Could not search tags. Try again.'; }
        finally { if (request === version) loading = false; }
    }
</script>
<div class="finder">
    <form onsubmit={(event) => { event.preventDefault(); void search(); }}><label>{label}<input bind:value={query} placeholder="Name or UUID" /></label><button disabled={loading || !query.trim()}>Search</button></form>
    {#if error}<p role="alert">{error}</p>{/if}
    <div class="results">{#each results.filter(tag => !exclude.includes(tag.id)) as tag (tag.id)}
        <button type="button" onclick={() => onSelect(tag)}><strong>{tag.name}</strong><small>{categoryName(tag.category)} · {tag.count} posts</small><code>{tag.id}</code></button>
    {/each}</div>
    {#if results.length === 50}<small>First 50 matches. Narrow your search to find more.</small>{/if}
</div>
<style>
    form { display: flex; align-items: end; gap: .5rem; } label { flex: 1; font-size: .85rem; } input { display: block; width: 100%; min-width: 0; margin-top: .3rem; }
    input, button { border: 1px solid #cbd5e1; border-radius: 8px; padding: .5rem .65rem; background: white; } button { cursor: pointer; } button:disabled { opacity: .5; }
    .results { display: grid; gap: .4rem; max-height: 300px; overflow: auto; margin-top: .6rem; } .results button { text-align: left; display: grid; gap: .15rem; } small { color: #64748b; } code { font-size: .65rem; overflow-wrap: anywhere; color: #64748b; } [role="alert"] { color: #b91c1c; }
</style>
