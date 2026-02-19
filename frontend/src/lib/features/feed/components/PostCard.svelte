<script lang="ts">
    import type { Post } from "$lib/domain/models/post";
    import { MediaType } from "$lib/domain";

    export let post: Post;

    $: fileUrl = `/api/files/${post.file.id}`;
    $: mediaType = String(post.file.media_type ?? "").toLowerCase();
    $: isPicture = mediaType === MediaType.Picture.toLowerCase();
    $: isVideo = mediaType === MediaType.Video.toLowerCase();
    $: isAudio = mediaType === MediaType.Audio.toLowerCase();
</script>

<div class="card">
    {#if isPicture}
        <img src={fileUrl} alt={post.title} loading="lazy" />
    {:else if isVideo}
        <video controls preload="metadata">
            <source src={fileUrl}>
        </video>
    {:else if isAudio}
        <audio controls preload="metadata">
            <source src={fileUrl}>
        </audio>
    {:else}
        <p>Unsupported Type</p>
    {/if}
    <div class="info">
        <h3>{post.title}</h3>
    </div>
</div>

<style>
    .card { border: 1px solid #ccc; border-radius: 8px; overflow: hidden; }
    img { width: 100%; height: 200px; object-fit: cover; }
    video { width: 100%; height: 200px; object-fit: cover; }
    .info { padding: 1em; }
</style>
