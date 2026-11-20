import type { RelationEvent } from './management';
export interface Edge { parent: string; child: string }
export function edgeKey(edge: Edge) { return `${edge.parent}:${edge.child}`; }
export function wouldCycle(edges: Edge[], parent: string, child: string): boolean {
    const seen = new Set<string>(); const pending = [child];
    while (pending.length) { const id = pending.pop()!; if (id === parent) return true; if (seen.has(id)) continue; seen.add(id); for (const edge of edges) if (edge.parent === id) pending.push(edge.child); }
    return false;
}
export function relationChanges(original: Edge[], current: Edge[]): RelationEvent[] {
    const before = new Set(original.map(edgeKey)), after = new Set(current.map(edgeKey));
    return [...original.filter(e => !after.has(edgeKey(e))).map(e => ({ op: 'unlink' as const, parent_id: e.parent, child_ids: [e.child] })),
        ...current.filter(e => !before.has(edgeKey(e))).map(e => ({ op: 'link' as const, parent_id: e.parent, child_ids: [e.child] }))];
}
