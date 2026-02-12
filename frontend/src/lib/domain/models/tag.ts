import type { UUID } from "$lib/domain/value-objects/uuid";

export interface Tag {
    id: UUID;
    category: TagCategory;
    value: string;
}

export enum TagCategory {
    Artist,
    Copyright,
    Character,
    General,
}
