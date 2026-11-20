<script lang="ts">
    import type { CollectionSummary } from '$lib/domain/models/collection';
    import { repositories } from '$lib/composition/repositories';
    import { createCollection } from '$lib/application/use-cases/create-collection';
    import { updateCollection } from '$lib/application/use-cases/update-collection';
    let { collection = null, deleting = false, onDone, onClose } = $props<{
        collection?: CollectionSummary | null; deleting?: boolean; onDone: () => void; onClose: () => void;
    }>();
    let title = $state('');
    let description = $state('');
    let saving = $state(false);
    let error = $state('');
    let initialized = false;
    $effect(() => { if (!initialized) { title = collection?.title ?? ''; description = collection?.description ?? ''; initialized = true; } });
    function modal(node: HTMLDialogElement) { node.showModal(); }
    async function submit() {
        saving = true; error = '';
        try {
            if (deleting && collection) await repositories.collections.deleteCollection(collection.id);
            else if (collection) await updateCollection(repositories.collections, collection.id, { title: title.trim(), description: description.trim() });
            else await createCollection(repositories.collections, { title: title.trim(), description: description.trim(), items: [] });
            onDone();
        } catch (err) { error = err instanceof Error ? err.message : 'Could not save collection. Please retry.'; }
        finally { saving = false; }
    }
</script>
<dialog use:modal oncancel={(event) => { event.preventDefault(); if (!saving) onClose(); }} aria-labelledby="collection-dialog-title">
    <form onsubmit={(event) => { event.preventDefault(); void submit(); }}>
        <h2 id="collection-dialog-title">{deleting ? 'Delete collection' : collection ? 'Edit collection' : 'New empty collection'}</h2>
        {#if deleting}
            <p>Delete “{collection?.title}” and all its collection items? Posts themselves will remain.</p>
        {:else}
            <label for="collection-name">Title</label><input id="collection-name" bind:value={title} required disabled={saving} />
            <label for="collection-description">Description</label><textarea id="collection-description" bind:value={description} rows="4" disabled={saving}></textarea>
        {/if}
        {#if error}<p class="error" role="alert">{error}</p>{/if}
        <div class="actions"><button type="button" onclick={onClose} disabled={saving}>Cancel</button><button class:danger={deleting} disabled={saving || (!deleting && !title.trim())}>{saving ? 'Saving…' : deleting ? 'Delete collection' : 'Save'}</button></div>
    </form>
</dialog>
<style>
    dialog { margin: auto; border: 1px solid #d1d5db; border-radius: 12px; padding: 1.5rem; width: min(520px, 95vw); max-height: 90vh; overflow: auto; }
    dialog::backdrop { background: #0007; }
    form { display: flex; flex-direction: column; gap: .7rem; }
    h2 { font-size: 1.2rem; font-weight: 600; } input, textarea { border: 1px solid #d1d5db; border-radius: 6px; padding: .6rem; }
    .actions { display: flex; justify-content: flex-end; gap: .5rem; }
    button { border: 1px solid #d1d5db; border-radius: 6px; padding: .5rem .8rem; cursor: pointer; }
    button:disabled { opacity: .5; cursor: default; } .danger { background: #b91c1c; color: white; } .error { color: #b91c1c; }
</style>
