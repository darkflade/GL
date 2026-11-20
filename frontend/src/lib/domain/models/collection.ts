import type { UUID } from "$lib/domain/value-objects/uuid";
import type { Tag } from "$lib/domain/models/tag";
import type { Post } from "$lib/domain/models/post";

export interface CollectionSummary {
    id: UUID
    title: string
    description: string
    cover: UUID | null
    item_count: number
    tags: Tag[]
}

export interface Collection {
    id: UUID
    title: string
    description: string | null
    tags: Tag[]
    cover: UUID | null
    items: CollectionItem[]
}

export interface CollectionItem {
    id: UUID
    position: number
    content: CollectionContent
}

export type CollectionContent =
    | { Post: Post }
    | { Note: string }
