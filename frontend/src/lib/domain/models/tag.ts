import type { UUID } from "$lib/domain/value-objects/uuid";

export interface Tag {
    id: UUID;
    category: TagCategory;
    name: string;
    count: number;
}

export interface TagRelation {
    id: UUID;
    parent_id: UUID;
    parent_name: string;
    parent_count: number;
    child_id: UUID;
    child_name: string;
    child_count: number;
    score: number;
}

export enum TagCategory {
    Artist,
    Copyright,
    Character,
    General,
}
