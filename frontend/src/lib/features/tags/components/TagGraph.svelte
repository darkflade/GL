<script lang="ts">
    import type { Tag } from '$lib/domain/models/tag';
    import { tagManagement, type Neighborhood } from '../management';
    import { edgeKey, relationChanges, wouldCycle, type Edge } from '../graph';
    import TagFinder from './TagFinder.svelte';
    import TagDropTarget from './TagDropTarget.svelte';
    let { details, onSaved, onFocus, onDirty } = $props<{ details: Neighborhood; onSaved: () => Promise<void>; onFocus: (id: string) => void; onDirty: (dirty: boolean) => void }>();
    type Node = { tag: Tag; x: number; y: number };
    let nodes = $state<Node[]>([]), edges = $state<Edge[]>([]), original = $state<Edge[]>([]);
    let initialized = false; let busy = $state(false); let error = $state(''); let canvas = $state<HTMLDivElement>();
    let drag = $state<{ id: string; dx: number; dy: number } | null>(null);
    let connecting = $state<string | null>(null); let pointer = $state({ x: 0, y: 0 });
    let parent = $state(''), child = $state('');
    let notice = $state('');
    const changes = $derived(relationChanges(original, edges));
    $effect(() => { onDirty(changes.length > 0); });
    const height = $derived(Math.max(720, ...nodes.map(node => node.y + 150)));
    $effect(() => { if (!initialized) { initialized = true; initialize(); } });
    function initialize() {
        nodes = [{ tag: details.tag, x: 370, y: 240 }];
        edges = [...details.children.map((tag: Tag) => ({ parent: details.tag.id, child: tag.id }))];
        original = edges.map(edge => ({ ...edge }));
        details.children.slice(0, 8).forEach((tag: Tag, i: number) => nodes.push({ tag, x: 25 + i % 4 * 235, y: 450 + Math.floor(i / 4) * 150 }));
    }
    async function addRelation(tag: Tag, kind: 'child' | 'alias') {
        if (busy) return;
        notice = '';
        if (tag.id === details.tag.id) { error = 'A tag cannot be its own child or alias.'; return; }
        if (kind === 'child') { addNode(tag); connect(details.tag.id, tag.id); return; }
        busy = true; error = '';
        try { await tagManagement.relations([{ op: 'alias', tag_id: details.tag.id, alias_ids: [tag.id] }]); notice = `Alias added: ${tag.name}`; }
        catch { error = 'Could not add alias. Try again.'; }
        finally { busy = false; }
    }
    function addNode(tag: Tag) {
        if (nodes.some(node => node.tag.id === tag.id)) return;
        nodes = [...nodes, { tag, x: 30 + nodes.length % 4 * 235, y: height - 120 }];
    }
    function connect(from: string, to: string) {
        if (!from || !to || busy) return;
        if (edges.some(edge => edge.parent === from && edge.child === to)) return;
        if (wouldCycle(edges, from, to)) { error = 'This connection creates a cycle. Choose a different parent.'; return; }
        error = ''; edges = [...edges, { parent: from, child: to }];
    }
    function position(event: PointerEvent) { const rect = canvas!.getBoundingClientRect(); return { x: event.clientX - rect.left, y: event.clientY - rect.top }; }
    function startMove(event: PointerEvent, node: Node) {
        if (event.button !== 0 || busy) return;
        event.preventDefault(); const p = position(event); drag = { id: node.tag.id, dx: p.x - node.x, dy: p.y - node.y };
    }
    function move(event: PointerEvent) {
        if (!canvas || (!drag && !connecting)) return;
        const p = position(event); pointer = p;
        if (drag) { const current = drag; nodes = nodes.map(node => node.tag.id === current.id ? { ...node, x: Math.max(0, Math.min(780, p.x - current.dx)), y: Math.max(0, Math.min(2400, p.y - current.dy)) } : node); }
    }
    function release(event: PointerEvent) {
        if (connecting) { const target = document.elementFromPoint(event.clientX, event.clientY)?.closest('[data-node]')?.getAttribute('data-node'); if (target) connect(connecting, target); }
        connecting = null; drag = null;
    }
    function path(edge: Edge): string {
        const from = nodes.find(node => node.tag.id === edge.parent), to = nodes.find(node => node.tag.id === edge.child);
        if (!from || !to) return '';
        return `M ${from.x + 100} ${from.y + 100} C ${from.x + 100} ${from.y + 160}, ${to.x + 100} ${to.y - 60}, ${to.x + 100} ${to.y}`;
    }
    function name(id: string) { return [...nodes.map(node => node.tag), ...details.parents, ...details.children].find(tag => tag.id === id)?.name ?? id; }
    function focus(id: string) { onFocus(id); }
    async function save() {
        busy = true; error = '';
        try { await tagManagement.relations(changes); original = edges.map(edge => ({ ...edge })); await onSaved(); }
        catch { error = 'Could not save graph. A cycle through tags outside this view may cause a conflict. Your changes are preserved.'; }
        finally { busy = false; }
    }
</script>
<svelte:window onpointermove={move} onpointerup={release} onpointercancel={() => { drag = null; connecting = null; }} />
<TagDropTarget onAdd={addRelation} disabled={busy} targetName={details.tag.name}>
<section class="graph-editor">
    <div class="graph-head"><div><h2>Hierarchy · {details.tag.name}</h2><p>Drag a node by its title. Drag its blue output onto another node to connect parent → child.</p></div><button class="primary" disabled={busy || !changes.length} onclick={save}>{busy ? 'Saving…' : `Save graph (${changes.length})`}</button></div>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    {#if notice}<p role="status">{notice}</p>{/if}
    <div class="graph-tools">
        <TagFinder label="Add tag to graph" exclude={nodes.map(node => node.tag.id)} onSelect={addNode} />
        <div class="keyboard"><label>Parent<select bind:value={parent}><option value="">Choose…</option>{#each nodes as node (node.tag.id)}<option value={node.tag.id}>{node.tag.name}</option>{/each}</select></label><label>Child<select bind:value={child}><option value="">Choose…</option>{#each nodes as node (node.tag.id)}<option value={node.tag.id}>{node.tag.name}</option>{/each}</select></label><button disabled={busy || !parent || !child} onclick={() => connect(parent, child)}>Connect →</button></div>
    </div>
    <div class="canvas-scroll"><div class="canvas" bind:this={canvas} style:height={`${height}px`}>
        <svg width="1000" {height} aria-hidden="true"><defs><marker id="tag-arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth"><path d="M0,0 L0,6 L9,3 z" fill="#64748b" /></marker></defs>
            {#each edges as edge (edgeKey(edge))}<path d={path(edge)} fill="none" stroke="#94a3b8" stroke-width="2" marker-end="url(#tag-arrow)" />{/each}
            {#if connecting}{@const node = nodes.find(node => node.tag.id === connecting)}{#if node}<path d={`M ${node.x + 100} ${node.y + 100} L ${pointer.x} ${pointer.y}`} stroke="#2563eb" stroke-width="3" stroke-dasharray="6 4" />{/if}{/if}
        </svg>
        {#each nodes as node (node.tag.id)}
            <div class="node" data-node={node.tag.id} class:root={node.tag.id === details.tag.id} style:left={`${node.x}px`} style:top={`${node.y}px`}>
                <button class="grip" onpointerdown={(event) => startMove(event, node)} title="Drag to move node">⠿ {node.tag.name}</button>
                <small>{node.tag.count} posts {node.tag.id === details.tag.id ? '· focus' : ''}</small>
                <div class="node-actions"><button disabled={busy} onclick={() => focus(node.tag.id)}>Explore</button><button class="output" aria-label={`Connect from ${node.tag.name}`} disabled={busy} onpointerdown={(event) => { event.preventDefault(); connecting = node.tag.id; pointer = position(event); }}>● →</button></div>
            </div>
        {/each}
    </div></div>
    <div class="more"><button disabled={busy} onclick={() => details.children.forEach(addNode)}>Show all children ({details.children.length})</button></div>
    <details><summary>Connections ({edges.length}) · remove branches</summary><div class="edge-list">{#each edges as edge (edgeKey(edge))}<div><span>{name(edge.parent)} → {name(edge.child)}</span><button disabled={busy} onclick={() => edges = edges.filter(item => edgeKey(item) !== edgeKey(edge))}>Unlink</button></div>{/each}</div></details>
</section>
</TagDropTarget>
<style>
    .graph-editor { display: grid; gap: 1rem; min-width: 0; } .graph-head { display: flex; align-items: center; justify-content: space-between; gap: 1rem; } h2 { font-size: 1.2rem; font-weight: 600; } p,small { color: #64748b; font-size: .85rem; } button,select { border: 1px solid #cbd5e1; border-radius: 7px; padding: .4rem .6rem; background: white; } button { cursor: pointer; } button:disabled { opacity: .5; cursor: default; } .primary { background: #2563eb; color: white; white-space: nowrap; }
    .graph-tools { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; } .keyboard { display: flex; gap: .5rem; align-items: end; flex-wrap: wrap; } label { display: grid; gap: .25rem; font-size: .85rem; } select { max-width: 180px; }
    .canvas-scroll { overflow: auto; max-height: 640px; border: 1px solid #e2e8f0; border-radius: 12px; } .canvas { width: 1000px; position: relative; background-color: #f8fafc; background-image: radial-gradient(#cbd5e1 1px, transparent 1px); background-size: 20px 20px; } svg { position: absolute; inset: 0; pointer-events: none; }
    .node { position: absolute; width: 200px; height: 100px; background: white; border: 1px solid #cbd5e1; border-radius: 10px; box-shadow: 0 3px 8px #0f172a12; padding: .45rem; display: grid; gap: .2rem; } .root { border: 2px solid #2563eb; background: #eff6ff; } .grip { border: 0; text-align: left; font-weight: 600; padding: .15rem; cursor: grab; touch-action: none; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .node-actions { display: flex; justify-content: space-between; } .node-actions button { font-size: .7rem; padding: .1rem .4rem; } .output { color: #2563eb; touch-action: none; }
    .more { display: flex; gap: .5rem; } .edge-list { display: grid; gap: .4rem; margin-top: .5rem; max-height: 300px; overflow: auto; } .edge-list div { display: flex; justify-content: space-between; align-items: center; gap: 1rem; } .error { color: #b91c1c; }
    @media(max-width:900px) { .graph-tools { grid-template-columns: 1fr; } .graph-head { align-items: start; flex-direction: column; } }
</style>
