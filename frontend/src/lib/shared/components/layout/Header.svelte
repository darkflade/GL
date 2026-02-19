<script lang="ts">
    import Button from '$lib/shared/components/ui/Button.svelte';
</script>
<div class="div">
    <nav>
        <Button href="/">Feed</Button>
        <Button href="/playlist">Playlists</Button>


        {#if 0}
            <span>Welcome, some dude!</span>
            <Button>Log Out</Button>
        {:else}
            <Button href="auth/login/">Sign In</Button>
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
    a {
        color: white;
    }
</style>
