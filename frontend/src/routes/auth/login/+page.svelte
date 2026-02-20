<script lang="ts">
    import type {UserInfo} from "$lib/domain/value-objects/auth";
    import { login } from "$lib/application/use-cases/authentication";
    import {repositories} from "$lib/composition/repositories";
    import Header from "$lib/shared/components/layout/Header.svelte";
    import Button from "$lib/shared/components/ui/Button.svelte";

    let userInfo: UserInfo = $state({
        username: "",
        password: ""
    })

    async function handleLogin(event: SubmitEvent) {
        event.preventDefault()

        const response = await login(repositories.authentication, userInfo)
        console.log(`[INFO] Login state: ${response}`)
    }

</script>
<Header/>

<div class="auth-page">
    <div class="glow-bg"></div>
    
    <div class="auth-container">
        <div class="auth-card glass-panel">
            <div class="auth-header">
                <h1 class="auth-title text-gradient">Welcome Back</h1>
                <p class="auth-subtitle">Sign in to Glab to continue your journey.</p>
            </div>

            <form class="auth-form" onsubmit={handleLogin}>
                <div class="form-group">
                    <label for="username">Nickname</label>
                    <input 
                        type="text" 
                        id="username"
                        placeholder="e.g. creative_mind" 
                        bind:value={userInfo.username} 
                        required 
                    />
                </div>
                
                <div class="form-group">
                    <label for="password">Password</label>
                    <input 
                        type="password" 
                        id="password"
                        placeholder="••••••••" 
                        bind:value={userInfo.password} 
                        required 
                    />
                </div>

                <div class="auth-actions">
                    <Button type="submit" variant="primary" size="lg" width="full">Sign In</Button>
                </div>
            </form>

            <div class="auth-footer">
                <p>New to Glab? <a href="/auth/register" class="link">Create an account</a></p>
            </div>
        </div>
    </div>
</div>

<style>
    .auth-page {
        min-height: calc(100vh - var(--header-height));
        display: flex;
        align-items: center;
        justify-content: center;
        padding: var(--spacing-xl) var(--spacing-lg);
        position: relative;
        overflow: hidden;
    }

    .glow-bg {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: 600px;
        height: 600px;
        background: radial-gradient(circle, var(--color-accent-glow) 0%, transparent 70%);
        opacity: 0.5;
        pointer-events: none;
        z-index: -1;
    }

    .auth-container {
        width: 100%;
        max-width: 480px;
        z-index: 10;
    }

    .auth-card {
        padding: var(--spacing-2xl) var(--spacing-xl);
        border-radius: var(--radius-xl);
    }

    .auth-header {
        text-align: center;
        margin-bottom: var(--spacing-2xl);
    }

    .auth-title {
        font-size: 2.5rem;
        font-weight: 800;
        letter-spacing: -0.04em;
        margin-bottom: var(--spacing-xs);
    }

    .auth-subtitle {
        color: var(--color-text-secondary);
        font-size: var(--text-base);
    }

    .auth-form {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-lg);
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }

    .form-group label {
        font-size: var(--text-sm);
        font-weight: 600;
        color: var(--color-text-secondary);
        margin-left: 4px;
    }

    .form-group input {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        padding: 0.75rem 1rem;
        color: var(--color-text-primary);
        font-family: var(--font-sans);
        font-size: var(--text-base);
        transition: all 0.3s var(--ease-out);
        outline: none;
    }

    .form-group input:focus {
        border-color: var(--color-accent);
        background: rgba(255, 255, 255, 0.06);
        box-shadow: 0 0 0 4px var(--color-accent-glow);
    }

    .auth-actions {
        margin-top: var(--spacing-md);
    }

    .auth-footer {
        margin-top: var(--spacing-2xl);
        text-align: center;
        font-size: var(--text-sm);
        color: var(--color-text-muted);
    }

    .link {
        color: var(--color-accent-vibrant);
        text-decoration: none;
        font-weight: 600;
        transition: color 0.2s ease;
    }

    .link:hover {
        color: var(--color-text-primary);
    }
</style>
