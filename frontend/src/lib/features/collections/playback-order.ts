import type { CollectionItem } from "$lib/domain/models/collection";
import type { PlaybackSettings } from "./playback";
export function mediaKind(item: CollectionItem): string {
    return 'Post' in item.content ? String(item.content.Post.file.media_type).toLowerCase() : 'note';
}
export function nextIndex(items: CollectionItem[], current: number, delta: number, settings: PlaybackSettings): number | null {
    for (let offset = 1; offset <= items.length; offset++) {
        const raw = current + offset * delta;
        if (!settings.loop_collection && (raw < 0 || raw >= items.length)) return null;
        const index = (raw % items.length + items.length) % items.length;
        const kind = mediaKind(items[index]);
        if ((kind === 'audio' && settings.skip_audio) || (kind === 'video' && settings.skip_video)) continue;
        return index;
    }
    return null;
}
