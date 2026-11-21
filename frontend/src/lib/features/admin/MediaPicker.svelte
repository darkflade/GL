<script lang="ts">
    import { onMount } from 'svelte';
    import { adminFiles, type MediaEntry } from './files';

    let { mode, onSelect, onCancel } = $props<{
        mode: 'directory' | 'file';
        onSelect: (path: string) => void;
        onCancel: () => void;
    }>();

    let dialog: HTMLDialogElement;
    let current = $state('');
    let selected = $state('');
    let entries = $state<MediaEntry[]>([]);
    let loading = $state(true);
    let error = $state('');
    let filter = $state('');
    const filtered = $derived(entries.filter(entry => entry.name.toLowerCase().includes(filter.toLowerCase())));

    async function browse(path: string) {
        loading = true;
        error = '';
        try {
            entries = await adminFiles.browse(path, mode === 'directory');
            current = path;
            selected = '';
            filter = '';
        } catch (err) {
            error = err instanceof Error ? err.message : 'Could not open the directory.';
        } finally {
            loading = false;
        }
    }

    function up() {
        const parent = current.split('/').slice(0, -1).join('/');
        void browse(parent === '/media' ? '' : parent);
    }

    onMount(() => {
        dialog.showModal();
        void browse('');
    });
</script>

<dialog bind:this={dialog} aria-labelledby="media-picker-title" oncancel={onCancel}>
    <h2 id="media-picker-title">{mode === 'directory' ? 'Choose an import folder' : 'Choose the file on the server'}</h2>
    <div class="location">
        <button type="button" disabled={!current || loading} onclick={up}>Up</button>
        <code>{current || 'Media folders'}</code>
    </div>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    {#if loading}
        <p role="status">Loading…</p>
    {:else}
        <label>Filter this folder <input bind:value={filter} /></label>
        <ul>
            {#each filtered as entry (entry.path)}
                <li>
                    <button type="button" class:selected={selected === entry.path}
                        aria-pressed={entry.directory ? undefined : selected === entry.path}
                        onclick={() => entry.directory ? browse(entry.path) : selected = entry.path}>
                        {entry.name}{entry.directory && current ? '/' : ''}
                    </button>
                </li>
            {/each}
        </ul>
        {#if !filtered.length}<p>No matching entries.</p>{/if}
    {/if}
    <div class="actions">
        <button type="button" onclick={onCancel}>Cancel</button>
        <button type="button" class="primary" disabled={loading || !(mode === 'directory' ? current : selected)}
            onclick={() => onSelect(mode === 'directory' ? current : selected)}>
            {mode === 'directory' ? 'Use this folder' : 'Use selected file'}
        </button>
    </div>
</dialog>

<style>
    dialog { margin: auto; width: min(36rem, calc(100vw - 2rem)); padding: 1.25rem; border: 1px solid #cbd5e1; border-radius: 12px; background: white; color: #0f172a; }
    dialog::backdrop { background: #0f172a80; }
    h2 { font-size: 1.15rem; font-weight: 650; margin-bottom: .8rem; }
    .location, .actions { display: flex; align-items: center; gap: .65rem; }
    .actions { justify-content: end; margin-top: 1rem; }
    code { overflow-wrap: anywhere; }
    label { display: grid; gap: .25rem; margin-top: .8rem; }
    input, button { padding: .45rem .65rem; border: 1px solid #cbd5e1; border-radius: 7px; background: white; }
    button { cursor: pointer; }
    button:disabled { cursor: default; opacity: .5; }
    ul { margin-top: .5rem; max-height: 42vh; overflow: auto; display: grid; gap: .2rem; }
    li button { width: 100%; text-align: left; border-color: transparent; overflow-wrap: anywhere; }
    li button:hover, .selected { background: #eff6ff; border-color: #93c5fd; }
    .primary { background: #1d4ed8; color: white; }
    .error { color: #b91c1c; margin: .5rem 0; }
</style>
