<script lang="ts">
    import type { Post } from "$lib/domain/models/post";
    import { MediaType } from "$lib/domain";

    let { post, size = null } = $props<{
        post: Post;
        size?: string | null;
    }>();

    let fullSize = $derived(size === "full");
    let fileUrl = $derived(`/api/files/${post.file.id}`);
    let mediaType = $derived(String(post.file.media_type ?? "").toLowerCase());
    let isPicture = $derived(mediaType === MediaType.Picture.toLowerCase());
    let isVideo = $derived(mediaType === MediaType.Video.toLowerCase());
    let isAudio = $derived(mediaType === MediaType.Audio.toLowerCase());
</script>

<div class="card-outer">
    <div class="card" class:full-size={fullSize}>
        <div class="media-container">
            {#if isPicture}
                <img
                    src={fileUrl}
                    alt={post.title}
                    loading="lazy"
                    class="media-content"
                />
            {:else if isVideo}
                <video 
                    controls={fullSize} 
                    preload="metadata" 
                    class="media-content" 
                    muted={!fullSize} 
                    loop={!fullSize} 
                    autoplay={!fullSize}
                >
                    <source src={fileUrl}>
                </video>
            {:else if isAudio}
                <div class="audio-placeholder">
                    <div class="audio-icon"></div>
                    <audio controls preload="metadata">
                        <source src={fileUrl}>
                    </audio>
                </div>
            {:else}
                <div class="unsupported">
                    <p>Format not supported</p>
                </div>
            {/if}
            
            {#if !fullSize}
                <div class="overlay">
                    <div class="overlay-content">
                        <h3 class="post-title">{post.title}</h3>
                        {#if post.tags.length > 0}
                            <div class="tags">
                                {#each post.tags.slice(0, 2) as tag}
                                    <span class="tag">#{tag.value}</span>
                                {/each}
                                {#if post.tags.length > 2}
                                    <span class="tag">+{post.tags.length - 2}</span>
                                {/if}
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .card-outer {
        break-inside: avoid;
        margin-bottom: var(--spacing-lg);
    }

    .card {
        position: relative;
        border-radius: var(--radius-lg);
        overflow: hidden;
        background-color: var(--color-surface);
        transition: all 0.5s var(--ease-out);
        transform: translateZ(0);
        box-shadow: var(--shadow-md);
        border: 1px solid var(--color-border);
    }

    .card:not(.full-size):hover {
        transform: translateY(-8px) scale(1.02);
        box-shadow: var(--shadow-xl);
        border-color: var(--color-border-bright);
        z-index: 10;
    }

    .media-container {
        position: relative;
        width: 100%;
        overflow: hidden;
    }

    .media-content {
        width: 100%;
        height: auto;
        display: block;
        object-fit: cover;
        transition: transform 0.8s var(--ease-out);
    }

    .card:hover .media-content {
        transform: scale(1.08);
    }
    
    .full-size .media-content {
        max-height: 85vh;
        object-fit: contain;
        background: #000;
    }

    .overlay {
        position: absolute;
        inset: 0;
        background: linear-gradient(to top, rgba(0,0,0,0.6) 0%, rgba(0,0,0,0.2) 40%, rgba(0,0,0,0) 100%);
        opacity: 0;
        transition: all 0.4s var(--ease-out);
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        padding: var(--spacing-lg);
        pointer-events: none;
        transform: translateY(10px);
    }

    .card:hover .overlay {
        opacity: 1;
        transform: translateY(0);
    }

    .overlay-content {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }

    .post-title {
        color: white;
        font-size: var(--text-lg);
        font-weight: 700;
        margin: 0;
        letter-spacing: -0.02em;
        text-shadow: 0 2px 4px rgba(0,0,0,0.3);
    }

    .tags {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }

    .tag {
        font-size: 10px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: white;
        background: rgba(255, 255, 255, 0.1);
        padding: 4px 10px;
        border-radius: var(--radius-full);
        backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .audio-placeholder, .unsupported {
        padding: var(--spacing-2xl);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-raised) 100%);
        min-height: 200px;
        gap: var(--spacing-md);
        color: var(--color-text-muted);
    }

    .audio-icon {
        width: 48px;
        height: 48px;
        background: var(--color-accent);
        mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='currentColor'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3' /%3E%3C/svg%3E") no-repeat center;
        opacity: 0.5;
    }
</style>

