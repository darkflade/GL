<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0" />
    <title>Glab Storage</title>
</svelte:head>

<div class="app-layout">
    {@render children()}
</div>

<style>
    /* Global layout styles */
    .app-layout {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        background-color: var(--color-background);
        color: var(--color-text-primary);
        font-family: var(--font-sans);
        overflow-x: hidden;
    }

    :global(html) {
        scroll-behavior: smooth;
    }

    :global(main) {
        flex: 1;
        width: 100%;
        max-width: 1600px;
        margin: 0 auto;
        padding-top: var(--header-height);
    }
</style>

