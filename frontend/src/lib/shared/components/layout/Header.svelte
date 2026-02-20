<script lang="ts">
    import { page } from '$app/state';
    import Button from '$lib/shared/components/ui/Button.svelte';
</script>

<header class="header-wrapper glass">
    <div class="header-container">
        <a href="/" class="logo">
            <div class="logo-icon"></div>
            <span class="logo-text text-gradient">Glab</span>
        </a>

        <nav class="nav-links">
            <a href="/" class="nav-link" class:active={page.url.pathname === '/' || page.url.pathname === '/feed'}>Feed</a>
            <a href="/playlist" class="nav-link" class:active={page.url.pathname === '/playlist'}>Playlists</a>
        </nav>

        <div class="auth-section">
            {#if false}
                <div class="user-profile">
                    <span class="user-name">Alex</span>
                    <Button variant="ghost" size="sm">Log Out</Button>
                </div>
            {:else}
                <Button href="/auth/login" variant="primary" size="md">Sign In</Button>
            {/if}
        </div>
    </div>
</header>

<style>
    .header-wrapper {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        height: var(--header-height);
        z-index: 100;
        display: flex;
        align-items: center;
        border-bottom: 1px solid var(--color-border);
    }

    .header-container {
        width: 100%;
        max-width: 1600px;
        margin: 0 auto;
        padding: 0 var(--spacing-lg);
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .logo {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        text-decoration: none;
    }

    .logo-icon {
        width: 32px;
        height: 32px;
        background: var(--color-accent);
        border-radius: var(--radius-md);
        position: relative;
        overflow: hidden;
    }

    .logo-icon::after {
        content: '';
        position: absolute;
        inset: 0;
        background: linear-gradient(135deg, rgba(255,255,255,0.4) 0%, rgba(255,255,255,0) 100%);
    }

    .logo-text {
        font-size: var(--text-xl);
        font-weight: 800;
        letter-spacing: -0.03em;
    }

    .nav-links {
        display: flex;
        align-items: center;
        gap: var(--spacing-xl);
        background: rgba(255, 255, 255, 0.03);
        padding: 6px;
        border-radius: var(--radius-full);
        border: 1px solid var(--color-border);
    }

    .nav-link {
        text-decoration: none;
        color: var(--color-text-secondary);
        font-size: var(--text-sm);
        font-weight: 600;
        padding: 8px 20px;
        border-radius: var(--radius-full);
        transition: all 0.2s var(--ease-out);
    }

    .nav-link:hover {
        color: var(--color-text-primary);
    }

    .nav-link.active {
        background: var(--color-surface-hover);
        color: var(--color-text-primary);
        box-shadow: var(--shadow-sm);
    }

    .auth-section {
        display: flex;
        align-items: center;
    }

    .user-profile {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
    }

    .user-name {
        font-size: var(--text-sm);
        font-weight: 500;
        color: var(--color-text-secondary);
    }

    @media (max-width: 640px) {
        .logo-text {
            display: none;
        }
        
        .nav-links {
            gap: var(--spacing-sm);
        }
    }
</style>

