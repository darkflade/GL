import { api, request } from "$lib/infrastructure/http/client";
export interface PlaybackSettings {
    shortcut_item_id?: string | null;
    interval_seconds: number;
    direction: 'forward' | 'backward';
    loop_collection: boolean;
    skip_audio: boolean;
    skip_video: boolean;
    mute_video_with_audio: boolean;
    audio_end: 'repeat' | 'next' | 'stop';
    video_end: 'repeat' | 'next' | 'stop';
}
export const defaults: PlaybackSettings = { shortcut_item_id: null, interval_seconds: 5, direction: 'forward', loop_collection: true,
    skip_audio: true, skip_video: false, mute_video_with_audio: true, audio_end: 'next', video_end: 'next' };
export interface PlaybackPreferences { item_id: string | null; global: PlaybackSettings; collection: PlaybackSettings | null }
export const playbackRepository = {
    savePosition: (id: string, item_id: string) => request<void>(`/playback/position?collection_id=${id}`, { method: 'PUT', body: JSON.stringify({ item_id }), keepalive: true }),
    get: (id: string) => api.get<PlaybackPreferences>(`/playback?collection_id=${id}`),
    save: (settings: PlaybackSettings, id?: string) => request<void>(`/playback${id ? `?collection_id=${id}` : ''}`, { method: 'PUT', body: JSON.stringify(settings) }),
    reset: (id: string) => api.delete<void>(`/playback?collection_id=${id}`),
};
