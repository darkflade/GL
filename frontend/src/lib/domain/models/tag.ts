import type { UUID } from "$lib/domain/value-objects/uuid";

export interface Tag {
    id: UUID;
    category: TagCategory;
    name: string;
    count: number;
}

export enum TagCategory {
    Artist,
    Copyright,
    Character,
    General,
}
