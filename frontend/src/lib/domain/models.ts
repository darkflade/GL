export type UUID = string;

export enum MediaType {
    Picture = 0,
    Video = 1,
    Audio = 2,
}

export interface Post {
    id: UUID
    title: string
    file: File
    tags: Tag[]
}

export interface File {
    id: string
    path: string
    hash: string
    mediaType: MediaType
    meta: FileMeta
    createdAt: Date
}

export interface FileMeta {
    height: number
    weight: number
    duration_ms: number
}

export interface Tag {
    id: UUID
    category: TagCategory
    value: string
}

enum TagCategory {
    Artist,
    Copyright,
    Character,
    General,
}

export interface PlaylistSummary {
    id: UUID
    title: string
    description: string
    cover: UUID | null
    item_count: number
    tags: Tag[]
}

export interface SearchPostsQuery {
    must: string[]
    should: string[]
    must_not: string[]
}
