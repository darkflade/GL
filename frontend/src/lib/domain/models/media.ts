import type { UUID } from "$lib/domain/value-objects/uuid";

export enum MediaType {
    Picture = 0,
    Video = 1,
    Audio = 2,
}

export interface MediaFile {
    id: UUID;
    path: string;
    hash: string;
    mediaType: MediaType;
    meta: FileMeta;
    createdAt: Date;
}

export interface FileMeta {
    height: number;
    weight: number;
    duration_ms: number;
}
