import type { CollectionItem, CollectionContent } from "$lib/domain/models/collection";
import type { CollectionItemEvent, NewCollectionItemContent } from "$lib/domain/value-objects/create";

export interface DraftItem {
    key: string;
    id?: string;
    content: CollectionContent;
}

export function itemContent(content: CollectionContent): NewCollectionItemContent {
    return "Post" in content
        ? { type: "post", post_id: content.Post.id }
        : { type: "note", text: content.Note };
}

export function collectionEvents(original: CollectionItem[], draft: DraftItem[]): CollectionItemEvent[] {
    const events: CollectionItemEvent[] = [];
    const retained = new Set(draft.flatMap(item => item.id ? [item.id] : []));
    const before = new Map(original.map(item => [item.id, item]));
    const order = original.filter(item => retained.has(item.id)).map(item => item.id);
    let anchor: string | null = null;
    for (const item of draft) {
        if (!item.id) continue;
        const index = order.indexOf(item.id);
        const desired = anchor === null ? 0 : order.indexOf(anchor) + 1;
        if (index !== desired) {
            events.push({ op: "move", item_id: item.id, after_id: anchor });
            order.splice(index, 1);
            order.splice(anchor === null ? 0 : order.indexOf(anchor) + 1, 0, item.id);
        }
        if (JSON.stringify(itemContent(before.get(item.id)!.content)) !== JSON.stringify(itemContent(item.content))) {
            events.push({ op: "edit", item_id: item.id, content: itemContent(item.content) });
        }
        anchor = item.id;
    }
    // New IDs are assigned by the server. Insert each run backwards after its
    // existing anchor so no event needs to refer to a newly created item.
    let pending: DraftItem[] = [];
    anchor = null;
    function flush() {
        for (const item of pending.reverse()) {
            events.push({ op: "add", after_id: anchor, content: itemContent(item.content) });
        }
        pending = [];
    }
    for (const item of draft) {
        if (item.id) { flush(); anchor = item.id; }
        else pending.push(item);
    }
    flush();
    for (const item of original) {
        if (!retained.has(item.id)) events.push({ op: "remove", item_id: item.id });
    }
    return events;
}
