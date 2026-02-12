import type { UUID } from "$lib/domain/";
import type { Tag } from "$lib/domain/";
import type { Post } from "$lib/domain/";

export interface PlaylistSummary {
    id: UUID
    title: string
    description: string
    cover: UUID | null
    item_count: number
    tags: Tag[]
}

export interface Playlist {
    id: string
    title: string
    description: string
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
    | { kind: "Post"; data: Post }
    | { kind: "Note"; data: string }