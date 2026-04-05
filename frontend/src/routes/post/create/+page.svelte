<script lang="ts">
    import { goto } from "$app/navigation";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import TagSearch from "$lib/shared/components/layout/TagSearch.svelte";
    import { createPost } from "$lib/application/use-cases/create-post";
    import { repositories } from "$lib/composition/repositories";
    import type { Tag } from "$lib/domain/models/tag";

    let title = $state("");
    let selectedTags = $state<Tag[]>([]);
    let selectedFile = $state<File | null>(null);
    let submitting = $state(false);
    let errorMessage = $state("");

    function handleTagChange(tags: Tag[]) {
        selectedTags = tags;
    }

    function handleFileChange(event: Event) {
        const target = event.currentTarget as HTMLInputElement;
        selectedFile = target.files?.[0] ?? null;
    }

    async function handleSubmit(event: SubmitEvent) {
        event.preventDefault();

        if (!selectedFile) {
            errorMessage = "Select a media file before submit.";
            return;
        }

        errorMessage = "";
        submitting = true;
        try {
            const postID = await createPost(repositories.posts, {
                title: title.trim(),
                tags: selectedTags.map((tag) => tag.name),
                file: selectedFile,
            });

            await goto(`/post?id=${encodeURIComponent(postID)}`, {
                keepFocus: true,
                replaceState: false,
                noScroll: false,
            });
        } catch (error) {
            console.error(error);
            errorMessage = error instanceof Error ? error.message : "Failed to create post.";
        } finally {
            submitting = false;
        }
    }
</script>

<div class="min-h-screen bg-gray-50 text-gray-900">
    <Header />
    <header class="bg-white sticky top-0 z-20 px-6 py-3 flex items-center shadow-sm">
        <h1 class="text-xl font-bold tracking-tight">Create Post</h1>
    </header>

    <main class="content">
        <form class="card" onsubmit={handleSubmit}>
            <label class="field">
                <span>Title</span>
                <input
                    type="text"
                    placeholder="Post title"
                    bind:value={title}
                    maxlength="120"
                    required
                />
            </label>

            <div class="field">
                <span>Tags</span>
                <TagSearch onChange={handleTagChange} />
                <p class="hint">
                    Selected: {selectedTags.length}
                </p>
            </div>

            <label class="field">
                <span>File</span>
                <input type="file" accept="image/*,video/*" onchange={handleFileChange} required />
                {#if selectedFile}
                    <p class="hint">
                        {selectedFile.name}
                    </p>
                {/if}
            </label>

            {#if errorMessage}
                <p class="error">{errorMessage}</p>
            {/if}

            <button type="submit" class="submit-btn" disabled={submitting}>
                {submitting ? "Creating..." : "Create Post"}
            </button>
        </form>
    </main>
</div>

<style>
    .content {
        display: flex;
        justify-content: center;
        padding: 1rem;
    }

    .card {
        width: min(680px, 100%);
        background: #fff;
        border: 1px solid #e5e7eb;
        border-radius: 12px;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .field span {
        font-weight: 600;
        color: #111827;
    }

    .field input[type="text"],
    .field input[type="file"] {
        border: 1px solid #d1d5db;
        border-radius: 8px;
        padding: 0.55rem 0.7rem;
        background: #fff;
    }

    .hint {
        margin: 0;
        color: #6b7280;
        font-size: 0.85rem;
    }

    .error {
        margin: 0;
        color: #b91c1c;
        font-weight: 600;
    }

    .submit-btn {
        border: 1px solid #111827;
        background: #111827;
        color: #fff;
        border-radius: 8px;
        padding: 0.55rem 0.9rem;
        width: fit-content;
        cursor: pointer;
    }

    .submit-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
