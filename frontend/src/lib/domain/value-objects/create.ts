import {TagCategory, type UUID} from "$lib/domain";

export interface NewTag {
    category: TagCategory
    value: string
}

export interface NewPost {
    title: string
    file_id: UUID
}