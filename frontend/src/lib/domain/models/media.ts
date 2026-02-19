import type { UUID } from "$lib/domain/value-objects/uuid";

export enum MediaType {
    Picture = "Picture",
    Video = "Video",
    Audio = "Audio",
}

export interface MediaFile {
    id: UUID;
    path: string;
    hash: string;
    //TODO make deserialize
    media_type: MediaType;
    meta: FileMeta;
    createdAt: Date;
}

export interface FileMeta {
    height: number;
    weight: number;
    duration_ms: number;
}
