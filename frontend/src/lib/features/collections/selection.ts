export function selectRange(order: string[], selected: Set<string>, anchor: string | null, key: string,
    modifiers: { shiftKey: boolean; ctrlKey: boolean; metaKey: boolean }): Set<string> {
    if (modifiers.shiftKey && anchor && order.includes(anchor)) {
        const from = order.indexOf(anchor), to = order.indexOf(key);
        const result = new Set(selected);
        for (const id of order.slice(Math.min(from, to), Math.max(from, to) + 1)) {
            if (selected.has(anchor)) result.add(id); else result.delete(id);
        }
        return result;
    }
    if (modifiers.ctrlKey || modifiers.metaKey) {
        const result = new Set(selected);
        if (result.has(key)) result.delete(key); else result.add(key);
        return result;
    }
    return selected.size === 1 && selected.has(key) ? new Set() : new Set([key]);
}

export function moveSelection<T extends { key: string }>(items: T[], selected: Set<string>, boundary: number): T[] {
    const moving = items.filter(item => selected.has(item.key));
    const before = items.slice(0, boundary).filter(item => !selected.has(item.key));
    const after = items.slice(boundary).filter(item => !selected.has(item.key));
    return [...before, ...moving, ...after];
}

export function parsePostLinks(text: string): string[] {
    const lines = text.split(/\s+/).filter(Boolean);
    if (!lines.length) throw new Error("Paste at least one post link.");
    return lines.map((line, index) => {
        let url: URL;
        try { url = new URL(line); } catch { throw new Error(`Link ${index + 1} is not a valid URL.`); }
        const id = url.searchParams.get("id");
        if (!['http:', 'https:'].includes(url.protocol) || url.pathname.replace(/\/$/, '') !== '/post' ||
            !id || !/^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/i.test(id)) {
            throw new Error(`Link ${index + 1} must be a post URL with a valid id.`);
        }
        return id;
    });
}
