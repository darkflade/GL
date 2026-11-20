<script lang="ts">
    import type { Snippet } from 'svelte';
    import type { Tag } from '$lib/domain/models/tag';
    let { children, onAdd, disabled = false, targetName }: { children: Snippet; onAdd: (tag: Tag, kind: 'child' | 'alias') => void; disabled?: boolean; targetName: string } = $props();
    let hovering = $state(false);
    const mime = 'application/x-gl-tag';
    function accepts(event: DragEvent) { return !disabled && !!event.dataTransfer?.types.includes(mime); }
    function over(event: DragEvent) {
        if (!accepts(event)) return;
        event.preventDefault(); event.dataTransfer!.dropEffect = 'link'; hovering = true;
    }
    function drop(event: DragEvent, kind: 'child' | 'alias') {
        if (!accepts(event)) return;
        event.preventDefault(); event.stopPropagation(); hovering = false;
        try {
            const tag = JSON.parse(event.dataTransfer!.getData(mime)) as Tag;
            if (typeof tag.id === 'string' && typeof tag.name === 'string' && typeof tag.count === 'number' && (typeof tag.category === 'string' || typeof tag.category === 'number')) onAdd(tag, kind);
        } catch { /* Ignore unrelated or invalid drag payloads. */ }
    }
</script>
<svelte:window ondragend={() => hovering = false} ondrop={() => hovering = false} />
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="drop-target" ondragover={over} ondragleave={(event) => { if (!event.currentTarget.contains(event.relatedTarget as Node)) hovering = false; }}>
    {@render children()}
    {#if hovering && !disabled}
        <div class="drop-overlay">
            <button type="button" ondragover={over} ondrop={(event) => drop(event, 'child')}><strong>Add children</strong><span>Add as a child of {targetName}</span></button>
            <button type="button" ondragover={over} ondrop={(event) => drop(event, 'alias')}><strong>Add alias</strong><span>Add as an alias of {targetName}</span></button>
        </div>
    {/if}
</div>
<style>
    .drop-target { position: relative; min-height: 260px; }
    .drop-overlay { position: absolute; inset: 0; z-index: 20; display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; padding: 1rem; background: #eff6ffed; border-radius: 12px; }
    button { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: .7rem; border: 2px dashed #60a5fa; border-radius: 12px; color: #1e40af; background: #ffffffb3; padding: 1rem; }
    button:hover, button:focus-visible { background: #dbeafe; border-color: #2563eb; }
    strong { font-size: 1.4rem; } span { overflow-wrap: anywhere; }
</style>
