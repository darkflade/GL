<script lang="ts">
    import Button from '$lib/shared/components/ui/Button.svelte';
</script>

<header class="header">
    <nav>
        <a href="/">Главная</a>
        <a href="/playlist">Плейлисты</a>


        {#if 0}
            <span>Welcome, some dude!</span>
            <Button>Log Out</Button>
        {:else}
            <Button>Войти</Button>
        {/if}
    </nav>
</header>

<style>
    .header {
        padding: 1rem;
        background: #1f2937;
        color: white;
    }
</style>
