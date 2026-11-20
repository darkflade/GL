import { TagCategory } from "$lib/domain/models/tag";
import type { UUID } from "$lib/domain/value-objects/uuid";

export interface NewTag {
    category: TagCategory
    value: string
}

export interface NewPost {
    title: string
    file_id: UUID
}

export interface CreatePostInput {
    title: string;
    tags: string[];
    file: File;
}

export type NewCollectionItemContent =
    | { type: "post"; post_id: UUID }
    | { type: "note"; text: string };

export interface NewCollectionItem {
    position: number;
    content: NewCollectionItemContent;
}

export interface NewCollection {
    title: string;
    description?: string | null;
    tag_ids?: UUID[] | null;
    cover?: UUID | null;
    items?: NewCollectionItem[] | null;
}

export type CollectionItemEvent =
    | {
          op: "add";
          after_id?: UUID | null;
          content: NewCollectionItemContent;
      }
    | {
          op: "edit";
          item_id: UUID;
          content: NewCollectionItemContent;
      }
    | {
          op: "remove";
          item_id: UUID;
      }
    | {
          op: "move";
          item_id: UUID;
          after_id?: UUID | null;
      };

export interface UpdateCollection {
    title?: string;
    description?: string | null;
    add_tag_ids?: UUID[] | null;
    remove_tag_ids?: UUID[] | null;
    cover?: UUID | null;
    item_events?: CollectionItemEvent[] | null;
}
