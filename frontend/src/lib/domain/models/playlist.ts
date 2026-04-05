import type { UUID } from "$lib/domain/value-objects/uuid";
import type { Tag } from "$lib/domain/models/tag";
import type { Post } from "$lib/domain/models/post";

export interface PlaylistSummary {
    id: UUID
    title: string
    description: string
    cover: UUID | null
    item_count: number
    tags: Tag[]
}

export interface Playlist {
    id: UUID
    title: string
    description: string | null
    tags: Tag[]
    cover: UUID | null
    items: PlaylistItem[]
}

export interface PlaylistItem {
    id: UUID
    position: number
    content: PlaylistContent
}

export type PlaylistContent =
    | { Post: Post }
    | { Note: string }
