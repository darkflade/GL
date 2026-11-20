<script lang="ts">
    import type { Tag } from '$lib/domain/models/tag';
    import { categories, categoryName, tagManagement, type Neighborhood } from '../management';
    import TagFinder from './TagFinder.svelte';
    let { details, onChanged } = $props<{ details: Neighborhood; onChanged: (id: string | null) => Promise<void> }>();
    let name = $state(''); let category = $state('General'); let busy = $state(false); let error = $state('');
    let mode = $state<'edit' | 'delete' | 'merge'>('edit'); let target = $state<Tag | null>(null); let initialized = false;
    $effect(() => { if (!initialized) { name = details.tag.name; category = categoryName(details.tag.category); initialized = true; } });
    async function save() {
        busy = true; error = '';
        try {
            if (mode === 'merge') {
                if (!target) return;
                await tagManagement.update([{ op: 'merge', source_id: details.tag.id, target_id: target.id }]);
                await onChanged(target.id);
            } else if (mode === 'delete') {
                await tagManagement.update([{ op: 'remove', tag_ids: [details.tag.id] }]); await onChanged(null);
            } else {
                await tagManagement.update([{ op: 'edit', tags: [{ id: details.tag.id, name: name.trim(), category }] }]);
                await onChanged(details.tag.id);
            }
        } catch (err) { error = err instanceof Error ? `Could not apply change: ${err.message}. A duplicate name/category or a relation cycle can cause a conflict.` : 'Could not save tag.'; }
        finally { busy = false; }
    }
</script>
<section>
    <h2>{details.tag.name}</h2><code>{details.tag.id}</code>
    <dl><div><dt>Category</dt><dd>{categoryName(details.tag.category)}</dd></div><div><dt>Posts</dt><dd>{details.tag.count}</dd></div><div><dt>Parents / children / aliases</dt><dd>{details.parents.length} / {details.children.length} / {details.aliases.length}</dd></div></dl>
    <div class="actions"><button class:active={mode === 'edit'} onclick={() => mode = 'edit'} disabled={busy}>Edit</button><button class:active={mode === 'merge'} onclick={() => mode = 'merge'} disabled={busy}>Merge into…</button><button class:active={mode === 'delete'} onclick={() => mode = 'delete'} disabled={busy}>Delete</button></div>
    <fieldset disabled={busy}>
        {#if mode === 'edit'}
            <label>Name<input bind:value={name} /></label><label>Category<select bind:value={category}>{#each categories as value}<option>{value}</option>{/each}</select></label>
            <button class="primary" disabled={!name.trim()} onclick={save}>Save tag</button>
        {:else if mode === 'delete'}
            <p>Delete “{details.tag.name}”? Its assignments, branches and aliases will be removed. Posts remain.</p><button class="danger" onclick={save}>Delete this tag</button>
        {:else}
            <p>Choose the tag to keep. Assignments are combined without duplicates; the source tag is deleted. Branches and aliases are transferred. Cycles cancel the entire merge.</p>
            <TagFinder label="Merge target" exclude={[details.tag.id]} onSelect={(tag) => target = tag} />
            {#if target}<div class="target"><strong>Keep: {target.name}</strong><code>{target.id}</code><p>{categoryName(target.category)} · {target.count} posts</p></div>{/if}
            <button class="danger" disabled={!target} onclick={save}>Merge and delete “{details.tag.name}”</button>
        {/if}
    </fieldset>
    {#if busy}<p role="status">Saving…</p>{/if}{#if error}<p class="error" role="alert">{error}</p>{/if}
</section>
<style>
    section { display: grid; gap: 1rem; } h2 { font-size: 1.35rem; font-weight: 650; } code { font-size: .8rem; overflow-wrap: anywhere; color: #64748b; }
    dl { display: grid; gap: .5rem; } dl div { display: flex; justify-content: space-between; gap: 1rem; } dt { color: #64748b; }
    fieldset { border: 0; padding: 0; display: grid; gap: .8rem; } label { display: grid; gap: .35rem; } input,select,button { border: 1px solid #cbd5e1; border-radius: 8px; padding: .5rem .7rem; background: white; }
    button { cursor: pointer; } button:disabled { opacity: .5; cursor: default; } .actions { display: flex; gap: .5rem; } .active { background: #eff6ff; border-color: #2563eb; } .primary { background: #2563eb; color: white; } .danger { color: #b91c1c; border-color: #fca5a5; } .error { color: #b91c1c; } .target { display: grid; gap: .3rem; padding: .8rem; background: #f8fafc; border-radius: 8px; }
</style>
