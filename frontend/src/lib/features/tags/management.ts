import { api } from '$lib/infrastructure/http/client';
import type { Tag } from '$lib/domain/models/tag';
export interface Neighborhood { roots: Tag[]; tag: Tag; parents: Tag[]; children: Tag[]; aliases: Tag[] }
export type RelationEvent = { op: 'link' | 'unlink'; parent_id: string; child_ids: string[] } | { op: 'alias' | 'unalias'; tag_id: string; alias_ids: string[] };
export const tagManagement = {
    get: (id: string) => api.get<Neighborhood>(`/tags/${id}/neighborhood`),
    update: (events: unknown[]) => api.patch<void>('/tags', { events }),
    relations: (events: RelationEvent[]) => api.patch<void>('/tags/relations', { events }),
};
export const categories = ['Artist', 'Copyright', 'Character', 'General'];
export function categoryName(category: string | number): string { return typeof category === 'number' ? categories[category] ?? 'General' : category; }
