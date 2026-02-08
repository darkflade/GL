<script lang="ts">
    import { authStore } from '$lib/stores/authStore';
    import Button from '$lib/components/ui/Button.svelte';
</script>

<header class="header">
    <nav>
        <a href="/">Главная</a>
        <a href="/playlist">Плейлисты</a>

        {#if $authStore}
            <span>Привет, {$authStore.username}!</span>
            <Button on:click={() => authStore.logout()}>Выйти</Button>
        {:else}
            <Button href="/login">Войти</Button>
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