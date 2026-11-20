<script lang="ts">
    import { onDestroy, tick } from 'svelte';
    import type { Collection, CollectionItem } from '$lib/domain/models/collection';
    import { defaults, playbackRepository, type PlaybackSettings } from '../playback';
    import { mediaKind, nextIndex } from '../playback-order';
    let { collection, start, onClose } = $props<{ collection: Collection; start?: number; onClose: () => void }>();
    let index = $state(0);
    let playing = $state(false);
    let settings = $state<PlaybackSettings>({ ...defaults });
    let globalSettings = $state<PlaybackSettings>({ ...defaults });
    let override = $state(false);
    let settingsOpen = $state(false);
    let scope = $state<'collection' | 'global'>('collection');
    let form = $state<PlaybackSettings>({ ...defaults });
    let loading = $state(true);
    let saving = $state(false);
    let error = $state('');
    let message = $state('');
    let initialized = false;
    let disposed = false;
    let ownsFullscreen = false;
    let jump = $state(1);
    let positionQueue: Promise<void> = Promise.resolve();
    let lastSaved: string | undefined;
    let closing = $state(false);
    async function closeViewer() {
        if (closing) return;
        closing = true;
        playing = false;
        audio?.pause(); video?.pause();
        try { if (!loading && item) await persistPosition(item.id); }
        catch { /* Debounced saves already show errors; closing must remain possible. */ }
        finally { onClose(); }
    }
    function persistPosition(itemId: string) {
        positionQueue = positionQueue.catch(() => {}).then(async () => {
            if (itemId === lastSaved) return;
            await playbackRepository.savePosition(collection.id, itemId);
            lastSaved = itemId;
        });
        return positionQueue;
    }
    $effect(() => {
        if (loading || !item) return;
        const id = item.id;
        jump = index + 1;
        const timer = setTimeout(() => { void persistPosition(id).catch(() => { message = 'Could not save presentation position. It will be retried on the next slide.'; }); }, 400);
        return () => clearTimeout(timer);
    });
    function jumpTo() {
        if (!Number.isInteger(jump) || jump < 1 || jump > collection.items.length) { message = 'Enter an item number within the collection.'; return; }
        message = ''; playing = false; index = jump - 1;
    }
    let video = $state<HTMLVideoElement>();
    let audio = $state<HTMLAudioElement>();
    let audioIndex = $state<number | null>(null);
    let audioPlaying = $state(false);
    const item = $derived(collection.items[index]);
    const kind = $derived(item ? mediaKind(item) : 'note');
    const shortcutIndex = $derived(collection.items.findIndex((entry: CollectionItem) => entry.id === settings.shortcut_item_id));
    const direction = $derived(settings.direction === 'forward' ? 1 : -1);
    const tracks = $derived(collection.items.map((item: CollectionItem, index: number) => ({ item, index })).filter(({ item }: {item: CollectionItem}) => mediaKind(item) === 'audio'));
    const track = $derived(audioIndex === null ? undefined : collection.items[audioIndex]);

    $effect(() => {
        if (initialized) return;
        initialized = true;
        index = start ?? 0;
        void loadSettings();
    });
    async function loadSettings() {
        try {
            const result = await playbackRepository.get(collection.id);
            if (disposed) return;
            if (start === undefined && result.item_id) {
                const savedIndex = collection.items.findIndex((item: CollectionItem) => item.id === result.item_id);
                if (savedIndex >= 0) index = savedIndex;
            }
            globalSettings = result.global; override = result.collection !== null;
            settings = { ...(result.collection ?? result.global) };
        } catch { if (!disposed) error = 'Could not load playback settings. Defaults are active; reopen to retry.'; }
        finally { if (!disposed) loading = false; }
    }
    function stopOnDestroy(node: HTMLMediaElement) { return { destroy() { node.pause(); } }; }
    async function startTrack(value: number) { audioIndex = value; await tick(); await playMedia(audio); }
    $effect(() => { if (audioIndex === null) audioPlaying = false; });
    function modal(node: HTMLDialogElement) { node.showModal(); }
    async function fullscreen() {
        try {
            if (!document.fullscreenElement) { await document.documentElement.requestFullscreen(); ownsFullscreen = true; }
            else if (ownsFullscreen) { await document.exitFullscreen(); ownsFullscreen = false; }
        } catch { message = 'Fullscreen is unavailable. Presentation still fills the window.'; }
    }
    onDestroy(() => {
        disposed = true;
        const current = collection.items[index];
        if (!loading && current) void persistPosition(current.id).catch(() => {});
        audio?.pause(); video?.pause();
        if (ownsFullscreen && document.fullscreenElement) void document.exitFullscreen().catch(() => {});
    });
    function step(delta: number) {
        const next = nextIndex(collection.items, index, delta, settings);
        if (next === null) { playing = false; message = 'End of playback: no more eligible items.'; return; }
        message = '';
        if (next === index && video) { video.currentTime = 0; void playMedia(video); }
        index = next;
    }
    function toggleSlideshow() {
        if (loading) return;
        message = '';
        if (!playing && ((kind === 'audio' && settings.skip_audio) || (kind === 'video' && settings.skip_video))) {
            const next = nextIndex(collection.items, index, direction, settings);
            if (next === null) { message = 'No items match the playback settings.'; return; }
            index = next;
        }
        playing = !playing;
    }
    $effect(() => {
        if (playing && !settingsOpen && ((kind === 'audio' && settings.skip_audio) || (kind === 'video' && settings.skip_video))) {
            const next = nextIndex(collection.items, index, direction, settings);
            if (next === null) { playing = false; message = 'No items match the playback settings.'; }
            else index = next;
        }
    });
    $effect(() => {
        if (!playing || settingsOpen || loading || !item || kind === 'video') return;
        const currentIndex = index;
        const delay = settings.interval_seconds * 1000;
        const delta = direction;
        const timer = setTimeout(() => { if (index === currentIndex) step(delta); }, delay);
        return () => clearTimeout(timer);
    });
    async function playMedia(element: HTMLMediaElement | undefined) {
        if (!element) return;
        try { await element.play(); }
        catch { message = 'Playback was blocked or the file could not be played. Use the media controls to start it.'; }
    }
    $effect(() => {
        if (!video) return;
        if (playing && !settingsOpen) void playMedia(video); else video.pause();
    });
    $effect(() => {
        if (kind === 'audio' && !settings.skip_audio && playing) audioIndex = index;
    });
    function videoEnded() {
        if (settings.video_end === 'repeat') { if (video) video.currentTime = 0; void playMedia(video); }
        else if (settings.video_end === 'next' && playing) step(direction);
        else playing = false;
    }
    function audioEnded() {
        if (settings.audio_end === 'repeat') { if (audio) audio.currentTime = 0; void playMedia(audio); }
        else if (settings.audio_end === 'next' && audioIndex !== null) {
            const ordered = direction === 1 ? tracks : [...tracks].reverse();
            const position = ordered.findIndex(({ index }: {index: number}) => index === audioIndex);
            const next = ordered[position + 1] ?? (settings.loop_collection ? ordered[0] : undefined);
            if (next && next.index !== audioIndex) audioIndex = next.index;
            else if (next) { if (audio) audio.currentTime = 0; void playMedia(audio); }
        }
    }
    function keydown(event: KeyboardEvent) {
        if (settingsOpen || (event.target as HTMLElement).closest('input,select,textarea,button,audio,video')) return;
        if (event.key === 'ArrowRight') { event.preventDefault(); step(1); }
        if (event.key === 'ArrowLeft') { event.preventDefault(); step(-1); }
        if (event.code === 'Space') { event.preventDefault(); toggleSlideshow(); }
    }
    function openSettings() { form = { ...settings }; scope = 'collection'; settingsOpen = true; error = ''; }
    function changeScope(value: 'collection' | 'global') { scope = value; form = { ...(value === 'global' ? globalSettings : settings) }; }
    async function saveSettings(reset = false) {
        saving = true; error = '';
        try {
            if (reset) await playbackRepository.reset(collection.id);
            else {
                if (!Number.isInteger(form.interval_seconds) || form.interval_seconds < 1 || form.interval_seconds > 3600) throw new Error('Interval must be 1–3600 seconds.');
                await playbackRepository.save(form, scope === 'collection' ? collection.id : undefined);
            }
            const result = await playbackRepository.get(collection.id);
            globalSettings = result.global; override = result.collection !== null;
            settings = { ...(result.collection ?? result.global) };
            settingsOpen = false;
        } catch (err) { error = err instanceof Error ? err.message : 'Could not save settings. Please retry.'; }
        finally { saving = false; }
    }
</script>

<dialog class="viewer" use:modal oncancel={(event) => { event.preventDefault(); if (settingsOpen) { if (!saving) settingsOpen = false; } else void closeViewer(); }} onkeydown={keydown} aria-label="Collection presentation">
    <header>
        <strong>{collection.title} · {index + 1} / {collection.items.length}</strong>
        <div class="actions">
            <button onclick={() => step(-1)}>← Previous</button>
            <button onclick={toggleSlideshow} disabled={loading}>{playing ? 'Pause slideshow' : 'Start slideshow'}</button>
            <button onclick={() => step(1)}>Next →</button>
            <button onclick={openSettings} disabled={loading}>Playback settings</button>
            <form class="actions" onsubmit={(event) => { event.preventDefault(); jumpTo(); }}><label>Item <input aria-label="Go to item number" type="number" min="1" max={collection.items.length} step="1" bind:value={jump} style="width: 5rem" /></label><button>Go</button></form>
            <button onclick={fullscreen}>Fullscreen</button>
            <button onclick={closeViewer} disabled={closing}>{closing ? 'Saving position…' : 'Close ×'}</button>
        </div>
    </header>
    <main>
        {#key item?.id}
            {#if item && 'Post' in item.content}
                {#if kind === 'picture'}
                    <img src={`/api/files/full/${item.content.Post.file.id}`} alt={item.content.Post.title} onerror={() => message = 'Could not load image. You can skip to the next item.'} />
                {:else if kind === 'video'}
                    <video use:stopOnDestroy bind:this={video} src={`/api/files/full/${item.content.Post.file.id}`} controls playsinline
                        muted={settings.mute_video_with_audio && audioPlaying} onended={videoEnded}
                        onerror={() => { playing = false; message = 'Could not play video. Skip it or use the media controls.'; }}></video>
                {:else if kind === 'audio'}
                    <div class="note"><h2>{item.content.Post.title}</h2><p>Audio track — use the player below.</p>
                        <button onclick={() => startTrack(index)}>Play audio</button></div>
                {:else}<p>Unsupported media. Use Next to skip.</p>{/if}
            {:else if item && 'Note' in item.content}<p class="note">{item.content.Note}</p>{/if}
        {/key}
    </main>
    {#if shortcutIndex >= 0}
        <button class="shortcut" onclick={() => { playing = false; index = shortcutIndex; message = ''; }} title="Jump to the item saved in playback settings">↪ Item {shortcutIndex + 1}</button>
    {/if}
    <footer>
        {#if loading}<p>Loading playback settings…</p>{/if}
        {#if message}<p role="status">{message}</p>{/if}
        {#if error && !settingsOpen}<p role="alert">{error}</p>{/if}
        {#if tracks.length}
            <label>Background audio
                <select value={audioIndex ?? ''} onchange={(event) => { const value = event.currentTarget.value; audioIndex = value === '' ? null : Number(value); }}>
                    <option value="">Off</option>
                    {#each tracks as entry (entry.item.id)}<option value={entry.index}>{'Post' in entry.item.content ? entry.item.content.Post.title || `Track ${entry.index + 1}` : ''}</option>{/each}
                </select>
            </label>
            {#key track?.id}
                {#if track && 'Post' in track.content}
                    <audio use:stopOnDestroy bind:this={audio} src={`/api/files/full/${track.content.Post.file.id}`} controls
                        onloadedmetadata={() => playMedia(audio)} onplay={() => audioPlaying = true} onpause={() => audioPlaying = false}
                        onended={() => { audioPlaying = false; audioEnded(); }} onerror={() => { audioPlaying = false; message = 'Could not load audio. Select another track.'; }}></audio>
                {/if}
            {/key}
        {/if}
        <small>← → navigate · Space slideshow · Esc close</small>
    </footer>
    {#if settingsOpen}
        <div class="settings-backdrop">
            <dialog class="settings" use:modal aria-label="Playback settings" oncancel={(event) => { event.preventDefault(); if (!saving) settingsOpen = false; }}>
                <h2>Playback settings</h2>
                <p>{override ? 'This collection uses its own settings.' : 'This collection inherits global settings.'}</p>
                <label>Save for <select value={scope} disabled={saving} onchange={(event) => changeScope(event.currentTarget.value as 'collection' | 'global')}><option value="collection">This collection</option><option value="global">All collections (global)</option></select></label>
                {#if scope === 'global' && override}<p>This collection keeps its override until you choose Use global settings.</p>{/if}
                <fieldset disabled={saving}>
                    {#if scope === 'collection'}
                        <label>Quick-jump button target<select bind:value={form.shortcut_item_id}><option value={null}>No shortcut</option>{#each collection.items as entry, number (entry.id)}<option value={entry.id}>{number + 1}. {'Post' in entry.content ? entry.content.Post.title || 'Untitled post' : entry.content.Note.slice(0, 60)}</option>{/each}</select></label>
                    {/if}
                    <label>Slide interval (seconds)<input type="number" min="1" max="3600" step="1" bind:value={form.interval_seconds} /></label>
                    <label>Direction<select bind:value={form.direction}><option value="forward">Forward</option><option value="backward">Backward</option></select></label>
                    <label><input type="checkbox" bind:checked={form.loop_collection} />Loop collection / audio queue</label>
                    <label><input type="checkbox" bind:checked={form.skip_audio} />Skip audio slides (background audio remains available)</label>
                    <label><input type="checkbox" bind:checked={form.skip_video} />Skip video</label>
                    <label><input type="checkbox" bind:checked={form.mute_video_with_audio} />Mute video while background audio plays</label>
                    <label>When audio ends<select bind:value={form.audio_end}><option value="repeat">Repeat track</option><option value="next">Next audio track</option><option value="stop">Stop audio</option></select></label>
                    <label>When video ends<select bind:value={form.video_end}><option value="repeat">Repeat video</option><option value="next">Next slide</option><option value="stop">Stop slideshow</option></select></label>
                </fieldset>
                {#if error}<p role="alert">{error}</p>{/if}
                <div class="actions"><button disabled={saving} onclick={() => settingsOpen = false}>Cancel</button><button disabled={saving} onclick={() => saveSettings()}>Save</button><button disabled={saving || !override} onclick={() => saveSettings(true)}>Use global settings</button></div>
            </dialog>
        </div>
    {/if}
</dialog>

<style>
    .shortcut { position: absolute; right: 1.25rem; bottom: 6rem; z-index: 2; padding: .85rem 1rem; border-radius: 999px; background: #2563eb; border-color: #60a5fa; box-shadow: 0 4px 20px #0008; }
    .viewer { position: fixed; inset: 0; margin: 0; max-width: none; max-height: none; width: 100vw; height: 100dvh; border: 0; padding: 0; background: #080b10; color: white; display: flex; flex-direction: column; }
    .viewer::backdrop { background: #080b10; }
    header, footer { padding: .65rem 1rem; flex-shrink: 0; background: #111827; display: flex; align-items: center; justify-content: space-between; gap: .5rem; flex-wrap: wrap; }
    main { flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center; overflow: auto; }
    main img, main video { width: 100%; height: 100%; object-fit: contain; }
    .note { white-space: pre-wrap; overflow-wrap: anywhere; max-width: 70ch; padding: 2rem; font-size: 1.4rem; }
    .actions { display: flex; gap: .4rem; flex-wrap: wrap; }
    button, select, input { background: #1f2937; color: white; border: 1px solid #64748b; padding: .4rem .6rem; border-radius: 6px; }
    button { cursor: pointer; } button:disabled { opacity: .5; cursor: default; }
    small { color: #cbd5e1; } footer audio { height: 34px; }
    .settings-backdrop { position: absolute; inset: 0; background: #000b; display: grid; place-items: center; padding: 1rem; }
    .settings::backdrop { background: #0009; }
    .settings { margin: auto; color: white; border: 0; background: #111827; padding: 1.5rem; border-radius: 12px; max-width: 620px; width: 100%; max-height: 95%; overflow: auto; display: grid; gap: 1rem; }
    fieldset { display: grid; gap: .8rem; border: 0; padding: 0; }
    label { display: flex; gap: .5rem; align-items: center; flex-wrap: wrap; }
    h2, p { margin: 0; } [role="alert"] { color: #fca5a5; }
</style>
