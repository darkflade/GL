<script lang="ts">
    import Button from '$lib/shared/components/ui/Button.svelte';
    import { page } from "$app/state";

    function getSignInHref(): string {
        const redirectTo = `${page.url.pathname}${page.url.search}`;
        return `/auth/login?redirect=${encodeURIComponent(redirectTo)}`;
    }
</script>
<div class="div">
    <nav>
        <Button href="/">Feed</Button>
        <Button href="/playlists">Playlists</Button>
        <Button href="/tags">Tags</Button>
        <Button href="/post/create">Create Post</Button>


        {#if 0}
            <span>Welcome, some dude!</span>
            <Button>Log Out</Button>
        {:else}
            <Button href={getSignInHref()}>Sign In</Button>
        {/if}
    </nav>
</div>

<style>
    .div {
        padding: 1rem;
        padding-inline: 2rem;
        background: #2e2e2e;
        color: white;
        border-bottom: 1rem solid #afb1b1;
        border-radius: 8px;
        margin: -.5rem -1rem .5rem -1rem;
    }
</style>
