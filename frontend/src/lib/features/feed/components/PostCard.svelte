<script lang="ts">
    import type { Post } from "$lib/domain/models/post";
    import { MediaType } from "$lib/domain";

    export let post: Post;
    export let size: string | null | undefined;

    const fullSize = size == "full"

    $: fileUrl = `/api/files/${post.file.id}`;
    $: mediaType = String(post.file.media_type ?? "").toLowerCase();
    $: isPicture = mediaType === MediaType.Picture.toLowerCase();
    $: isVideo = mediaType === MediaType.Video.toLowerCase();
    $: isAudio = mediaType === MediaType.Audio.toLowerCase();
</script>

<div class="card">
    {#if isPicture}
        <img
            src={fileUrl}
            alt={post.title}
            title={post.tags.map(tag => tag.name).sort().join("|")}
            loading="lazy"
            class:is-full={fullSize}
        />
    {:else if isVideo}
        <video controls preload="metadata" class:is-full={fullSize}>
            <source src={fileUrl}>
        </video>
    {:else if isAudio}
        <audio controls preload="metadata">
            <source src={fileUrl}>
        </audio>
    {:else}
        <p>Unsupported Type</p>
    {/if}
</div>

<style>
    img { width: 100%; height: 200px; object-fit: cover; display: block; }
    video { width: 100%; height: 200px; object-fit: cover; display: block; }
    .is-full { height: 100% !important; }
            .card { border: 1px solid #ccc; border-radius: 8px; overflow: hidden; line-height: 0; }
</style>
