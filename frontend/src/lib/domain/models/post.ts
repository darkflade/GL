import type { UUID } from "$lib/domain/value-objects/uuid";
import type { MediaFile } from "$lib/domain/models/media";
import type { Tag } from "$lib/domain/models/tag";

export interface Post {
    id: UUID;
    title: string;
    description: string;
    file: MediaFile;
    tags: Tag[];
    notes: PostNote[];
}

export interface PostNote {
    id: string;
    text: string;
    x: number;
    y: number;
}
