<script lang="ts">
    import type { Collection, CollectionContent, CollectionItem } from "$lib/domain/models/collection";
    import type { Post } from "$lib/domain/models/post";
    import type { KeysetCursorDto } from "$lib/infrastructure/repositories/dto";
    import { repositories } from "$lib/composition/repositories";
    import { searchPosts } from "$lib/application/use-cases/search-posts";
    import { updateCollection } from "$lib/application/use-cases/update-collection";
    import { getPost } from "$lib/application/use-cases/get-post";
    import { selectRange, moveSelection, parsePostLinks } from "../selection";
    import { getCollection } from "$lib/application/use-cases/get-collection";
    import { ApiError } from "$lib/infrastructure/http/client";
    import { collectionEvents, type DraftItem } from "../edit-collection";
    import PostCard from "$lib/features/feed/components/PostCard.svelte";

    let { collection, onSaved, onCancel } = $props<{
        collection: Collection;
        onSaved: (collection: Collection) => void;
        onCancel: () => void;
    }>();
    // This component is mounted afresh for every editing session.
    let draft = $state<DraftItem[]>([]);
    let initialized = false;
    $effect(() => {
        if (!initialized) {
            draft = collection.items.map((item: CollectionItem) => ({ key: item.id, id: item.id, content: item.content }));
            initialized = true;
        }
    });
    let saving = $state(false);
    let error = $state("");
    let applied = $state(false);
    let selected = $state(new Set<string>());
    let anchor = $state<string | null>(null);
    let dragging = $state(false);
    let dropAt = $state<number | null>(null);
    let picker = $state<{ index: number; replace: boolean; targets: string[] } | null>(null);
    let chosen = $state(new Set<string>());
    let chosenPosts = $state<Post[]>([]);
    let postAnchor = $state<string | null>(null);
    let links = $state("");
    let importing = $state(false);
    let importError = $state("");
    let query = $state("");
    let posts = $state<Post[]>([]);
    let cursor = $state<KeysetCursorDto | undefined>();
    let searching = $state(false);
    let searchError = $state("");
    let searchVersion = 0;
    let activeQuery = "";
    let note = $state("");
    let contentType = $state<"post" | "note" | "import">("post");
    const changes = $derived(collectionEvents(collection.items, draft));

    function modal(node: HTMLDialogElement) { node.showModal(); }

    function selectItem(event: MouseEvent, key: string) {
        if (saving || applied) return;
        selected = selectRange(draft.map(item => item.key), selected, anchor, key, event);
        if (!event.shiftKey) anchor = key;
    }

    function selectPost(event: MouseEvent, post: Post) {
        chosen = selectRange(posts.map(item => item.id), chosen, postAnchor, post.id, event);
        const available = new Map([...chosenPosts, ...posts].map(item => [item.id, item]));
        chosenPosts = [...chosen].map(id => available.get(id)!);
        if (!event.shiftKey) postAnchor = post.id;
    }

    function removeSelected() {
        draft = draft.filter(item => !selected.has(item.key));
        selected = new Set(); anchor = null;
    }

    function startDrag(event: DragEvent, key: string) {
        if (saving || applied) { event.preventDefault(); return; }
        if (!selected.has(key)) { selected = new Set([key]); anchor = key; }
        dragging = true;
        if (event.dataTransfer) {
            event.dataTransfer.effectAllowed = "move";
            event.dataTransfer.setData("text/plain", key);
        }
    }

    function boundaryAt(event: DragEvent, index: number): number {
        const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
        return event.clientY < rect.top + rect.height / 2 ? index : index + 1;
    }

    function drop(event: DragEvent, boundary: number) {
        if (!dragging || saving || applied) return;
        event.preventDefault();
        draft = moveSelection(draft, selected, boundary);
        dragging = false; dropAt = null;
    }

    function openPicker(index: number, replace = false, targets?: string[]) {
        picker = { index, replace, targets: targets ?? (replace ? [draft[index].key] : []) };
        const content = replace ? draft.find(item => item.key === picker?.targets[0])?.content : null;
        contentType = content && "Note" in content ? "note" : "post";
        note = content && "Note" in content ? content.Note : "";
        chosen = new Set(); chosenPosts = []; postAnchor = null;
        links = ""; importError = "";
        query = "";
        void findPosts();
    }

    async function findPosts(more = false) {
        const version = ++searchVersion;
        searching = true;
        searchError = "";
        if (!more) { activeQuery = query; posts = []; cursor = undefined; }
        try {
            const result = await searchPosts(repositories.posts, {
                text_query: activeQuery,
                tag_query: { must: [], should: [], must_not: [] },
                cursor: more && cursor ? cursor : { mode: "keyset", limit: 20 },
            });
            if (version !== searchVersion) return;
            posts = more ? [...posts, ...result.posts] : result.posts;
            cursor = "has_next" in result && result.has_next ? result.next_cursor : undefined;
        } catch {
            if (version === searchVersion) searchError = "Failed to search posts. Try again.";
        } finally {
            if (version === searchVersion) searching = false;
        }
    }

    function choose(contents: CollectionContent[]) {
        if (!picker || contents.length === 0) return;
        if (picker.replace) {
            const targets = picker.targets;
            if (contents.length !== 1 && contents.length !== targets.length) return;
            draft = draft.map(item => {
                const index = targets.indexOf(item.key);
                return index < 0 ? item : { ...item, content: contents.length === 1 ? contents[0] : contents[index] };
            });
        } else {
            const items = [...draft];
            items.splice(picker.index, 0, ...contents.map(content => ({ key: crypto.randomUUID(), content })));
            draft = items;
        }
        picker = null;
    }

    async function importLinks() {
        importing = true; importError = "";
        try {
            const ids = parsePostLinks(links);
            const imported: Post[] = [];
            for (let start = 0; start < ids.length; start += 8) {
                const results = await Promise.allSettled(ids.slice(start, start + 8).map(id => getPost(repositories.posts, id)));
                for (const [offset, result] of results.entries()) {
                    if (result.status === "rejected") throw new Error(`Could not load post from link ${start + offset + 1}. No items were imported.`);
                    imported.push(result.value);
                }
            }
            choose(imported.map(post => ({ Post: post })));
        } catch (err) { importError = err instanceof Error ? err.message : "Import failed."; }
        finally { importing = false; }
    }

    async function save() {
        saving = true;
        error = "";
        try {
            if (!applied) {
                await updateCollection(repositories.collections, collection.id, { item_events: changes });
                applied = true;
            }
            // If refresh fails after PATCH, retry only the GET to avoid duplicate additions.
            onSaved(await getCollection(repositories.collections, collection.id));
        } catch (err) {
            error = applied ? "Changes saved, but refresh failed. Retry to reload the collection."
                : err instanceof ApiError && err.code === 401 ? "Session expired. Sign in before saving."
                : err instanceof ApiError && (err.code === 403 || err.code === 404) ? "Collection is unavailable or you do not have permission to edit it."
                : "Could not save changes. Your draft is preserved; please try again.";
        } finally { saving = false; }
    }
</script>

<div class="editor">
    <div class="toolbar">
        <div><h3>Edit items</h3><p>Changes are saved together. Click to select, Ctrl/Cmd to toggle, Shift for a range. Drag selected items to reorder.</p></div>
        <div class="actions">
            <button type="button" onclick={onCancel} disabled={saving || applied}>Cancel</button>
            <button type="button" class="primary" onclick={save} disabled={saving || (!applied && changes.length === 0)}>
                {saving ? "Saving…" : applied ? "Reload collection" : "Save changes"}
            </button>
        </div>
    </div>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <fieldset disabled={saving || applied}>
        <div class="actions selection-bar">
            <span>{selected.size} selected</span>
            <button type="button" onclick={() => selected = new Set(draft.map(item => item.key))}>Select all</button>
            <button type="button" disabled={!selected.size} onclick={() => { selected = new Set(); anchor = null; }}>Deselect all</button>
            <button type="button" disabled={!selected.size} onclick={() => openPicker(0, true, draft.filter(item => selected.has(item.key)).map(item => item.key))}>Replace selected</button>
            <button type="button" class="danger" disabled={!selected.size} onclick={removeSelected}>Delete selected</button>
            <button type="button" disabled={!selected.size} onclick={() => draft = moveSelection(draft, selected, 0)}>Move to beginning</button>
            <button type="button" disabled={!selected.size} onclick={() => draft = moveSelection(draft, selected, draft.length)}>Move to end</button>
        </div>
        <button type="button" class="insert" class:drop-target={dropAt === 0}
            ondragover={(event) => { if (dragging) { event.preventDefault(); dropAt = 0; } }}
            ondrop={(event) => drop(event, 0)} onclick={() => openPicker(0)}>+ Add at beginning / drop here</button>
        {#if draft.length === 0}<p class="muted">No items yet. Add a post or a note.</p>{/if}
        {#each draft as item, index (item.key)}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions (Selection is also available through the keyboard-accessible button.) -->
            <article class="item" class:selected={selected.has(item.key)} draggable={!saving && !applied}
                class:drop-before={dragging && dropAt === index} class:drop-after={dragging && dropAt === index + 1}
                ondragover={(event) => { if (dragging) { event.preventDefault(); dropAt = boundaryAt(event, index); } }}
                ondrop={(event) => drop(event, boundaryAt(event, index))}
                ondragstart={(event) => startDrag(event, item.key)}
                ondragend={() => { dragging = false; dropAt = null; }}
                onclick={(event) => { if (!(event.target as HTMLElement).closest('button, a, input, textarea, video, audio')) selectItem(event, item.key); }}>
                <button type="button" class="select-item" aria-pressed={selected.has(item.key)}
                    onclick={(event) => selectItem(event, item.key)}>
                    ⠿ {selected.has(item.key) ? "Selected" : "Select"} item {index + 1} · drag to move
                </button>
                <div class="item-head">
                    <span class="number">{index + 1}</span>
                    <span>{"Post" in item.content ? item.content.Post.title || "Untitled post" : "Note"}</span>
                    <div class="actions">
                        <button type="button" onclick={() => openPicker(index, true)}>{"Note" in item.content ? "Edit / replace" : "Replace"}</button>
                        <button type="button" class="danger" aria-label={`Remove item ${index + 1}`} onclick={() => { draft = draft.filter(row => row.key !== item.key); selected = new Set([...selected].filter(key => key !== item.key)); }}>Remove</button>
                    </div>
                </div>
                {#if "Post" in item.content}
                    <div class="preview"><PostCard post={item.content.Post} /></div>
                {:else}<p class="note">{item.content.Note}</p>{/if}
            </article>
            <button type="button" class="insert" class:drop-target={dropAt === index + 1}
                ondragover={(event) => { if (dragging) { event.preventDefault(); dropAt = index + 1; } }}
                ondrop={(event) => drop(event, index + 1)} onclick={() => openPicker(index + 1)}>+ Add here / drop here</button>
        {/each}
    </fieldset>
</div>

{#if picker}
    <div class="overlay">
        <dialog class="picker" use:modal aria-labelledby="picker-title" oncancel={(event) => { if (importing) event.preventDefault(); else picker = null; }}>
            <div class="item-head">
                <h3 id="picker-title">{picker.replace ? "Replace item" : "Add item"}</h3>
                <button type="button" disabled={importing} onclick={() => picker = null}>Close</button>
            </div>
            <fieldset disabled={importing}><div class="actions">
                <button type="button" class:chosen={contentType === "post"} onclick={() => contentType = "post"}>Post</button>
                <button type="button" class:chosen={contentType === "note"} onclick={() => contentType = "note"}>Note</button>
                {#if !picker.replace}<button type="button" class:chosen={contentType === "import"} onclick={() => contentType = "import"}>Import</button>{/if}
            </div>
            {#if picker.replace}<p>Replacing {picker.targets.length} items. Select one post for all, or the same number of posts for replacement in collection order.</p>{/if}
            {#if contentType === "import"}
                <label for="post-links">Post links, one per line (any domain)</label>
                <textarea id="post-links" rows="8" bind:value={links} placeholder="https://example.com/post?id=…"></textarea>
                {#if importError}<p class="error" role="alert">{importError}</p>{/if}
                <button type="button" class="primary" disabled={importing || !links.trim()} onclick={importLinks}>{importing ? "Importing…" : "Import in link order"}</button>
            {:else if contentType === "note"}
                <label for="item-note">Note text</label>
                <textarea id="item-note" rows="7" bind:value={note}></textarea>
                <button type="button" class="primary" disabled={!note.trim()} onclick={() => choose([{ Note: note }])}>Use note</button>
            {:else}
                <form onsubmit={(event) => { event.preventDefault(); void findPosts(); }}>
                    <label for="post-search">Search posts</label>
                    <div class="actions"><input id="post-search" bind:value={query} placeholder="Title or description" /><button disabled={searching}>Search</button></div>
                </form>
                {#if searchError}<p class="error" role="alert">{searchError}</p>{/if}
                <p>Ctrl/Cmd: toggle · Shift: range. {chosen.size} selected.</p>
                <div class="actions"><button type="button" onclick={() => { chosen = new Set(); chosenPosts = []; postAnchor = null; }}>Deselect all</button></div>
                <div class="results">
                    {#each posts as post (post.id)}
                        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions (The selection button provides keyboard access.) -->
                        <div class="result" class:selected={chosen.has(post.id)} onclick={(event) => { if (!(event.target as HTMLElement).closest('button, video, audio')) selectPost(event, post); }}><PostCard {post} /><button type="button" aria-pressed={chosen.has(post.id)} onclick={(event) => selectPost(event, post)}>{chosen.has(post.id) ? "✓ " : ""}{post.title || "Untitled post"}</button></div>
                    {/each}
                </div>
                {#if searching}<p role="status">Loading posts…</p>
                {:else if posts.length === 0 && !searchError}<p>No posts found.</p>{/if}
                {#if cursor}<button type="button" disabled={searching} onclick={() => findPosts(true)}>Load more</button>{/if}
                <button type="button" class="primary" disabled={!chosen.size || (picker.replace && chosen.size !== 1 && chosen.size !== picker.targets.length)}
                    onclick={() => choose(chosenPosts.map(post => ({ Post: post })))}>{picker.replace ? "Replace with selected posts" : "Add selected posts"} ({chosen.size})</button>
            {/if}
            </fieldset>
        </dialog>
    </div>
{/if}

<style>
    .editor { max-width: 960px; margin: auto; padding: 1rem; }
    .toolbar, .item-head { display: flex; align-items: center; justify-content: space-between; gap: .75rem; flex-wrap: wrap; }
    .toolbar { position: sticky; top: 0; z-index: 25; background: #f9fafb; padding: 1rem 0; }
    h3, p { margin: 0; }
    .toolbar p, .muted { color: #6b7280; font-size: .9rem; margin-top: .4rem; }
    .actions { display: flex; gap: .5rem; align-items: center; flex-wrap: wrap; }
    button { border: 1px solid #d1d5db; border-radius: 8px; padding: .45rem .75rem; background: white; cursor: pointer; }
    button:disabled { opacity: .45; cursor: default; }
    .primary { background: #2563eb; border-color: #2563eb; color: white; }
    .chosen { background: #eff6ff; border-color: #2563eb; }
    .danger, .error { color: #b91c1c; }
    .error { margin: .75rem 0; }
    fieldset { border: 0; padding: 0; min-width: 0; }
    .item { background: white; border: 1px solid #e5e7eb; border-radius: 12px; padding: 1rem; margin-top: 1rem; }
    .selected { outline: 2px solid #2563eb; background: #eff6ff; }
    .drop-before { box-shadow: 0 -4px #2563eb; }
    .drop-after { box-shadow: 0 4px #2563eb; }
    .select-item { width: 100%; text-align: left; margin-bottom: .75rem; cursor: grab; user-select: none; }
    .select-item:active { cursor: grabbing; }
    .selection-bar { margin-bottom: 1rem; }
    .drop-target { background: #dbeafe; outline: 3px solid #2563eb; }
    .insert { width: 100%; min-height: 40px; border-style: dashed; }
    .picker fieldset { display: flex; flex-direction: column; gap: 1rem; }
    .number { color: #6b7280; font-variant-numeric: tabular-nums; }
    .preview { max-width: 260px; margin-top: .75rem; }
    .note { white-space: pre-wrap; overflow-wrap: anywhere; padding: 1rem 0; }
    .insert { margin-top: .5rem; }
    .overlay { position: fixed; inset: 0; background: #0007; z-index: 80; display: grid; place-items: center; padding: 1rem; }
    .picker::backdrop { background: #0007; }
    .picker { margin: auto; border: 0; background: white; border-radius: 12px; padding: 1.25rem; width: min(720px, 100%); max-height: 90vh; overflow: auto; display: flex; flex-direction: column; gap: 1rem; }
    input, textarea { border: 1px solid #d1d5db; border-radius: 8px; padding: .6rem; width: 100%; min-width: 0; }
    input { flex: 1; }
    label { display: block; margin-bottom: .4rem; }
    .results { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: .75rem; }
    .result { min-width: 0; overflow-wrap: anywhere; }
    .result button { width: 100%; margin-top: .35rem; }
</style>
