<script lang="ts">
    import type {Post} from "$lib/domain/models";

    export let post: Post

    $: imageUrl = `api/files/${post.file.id}`
</script>

<div class="card">
    <img src={imageUrl} alt={post.title} loading="lazy" />
    <div class="info">
        <h3>{post.title}</h3>
    </div>
</div>

<style>
    .card { border: 1px solid #ccc; border-radius: 8px; overflow: hidden; }
    img { width: 100%; height: 200px; object-fit: cover; }
    .info { padding: 1em; }
</style>