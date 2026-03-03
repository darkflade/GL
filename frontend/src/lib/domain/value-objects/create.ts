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

export type NewPlaylistItemContent =
    | { type: "post"; post_id: UUID }
    | { type: "note"; text: string };

export interface NewPlaylistItem {
    position: number;
    content: NewPlaylistItemContent;
}

export interface NewPlaylist {
    title: string;
    description?: string | null;
    tag_ids?: UUID[] | null;
    cover?: UUID | null;
    items?: NewPlaylistItem[] | null;
}

export type PlaylistItemEvent =
    | {
          op: "add";
          after_id?: UUID | null;
          content: NewPlaylistItemContent;
      }
    | {
          op: "edit";
          item_id: UUID;
          content: NewPlaylistItemContent;
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

export interface UpdatePlaylist {
    title?: string;
    description?: string | null;
    add_tag_ids?: UUID[] | null;
    remove_tag_ids?: UUID[] | null;
    cover?: UUID | null;
    item_events?: PlaylistItemEvent[] | null;
}
