<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import type { UserInfo } from "$lib/domain/value-objects/auth";
    import { login } from "$lib/application/use-cases/authentication";
    import { repositories } from "$lib/composition/repositories";

    let userInfo: UserInfo = $state({
        username: "",
        password: "",
    });

    let pending = $state(false);
    let errorMessage = $state("");
    let registeredMessage = $derived(page.url.searchParams.get("registered") === "1");

    function resolveRedirect(target: string | null): string {
        if (!target || !target.startsWith("/") || target.startsWith("//")) {
            return "/feed";
        }
        return target;
    }

    function getRegisterHref(): string {
        const redirect = resolveRedirect(page.url.searchParams.get("redirect"));
        return `/auth/register?redirect=${encodeURIComponent(redirect)}`;
    }

    async function handleLogin(event: SubmitEvent) {
        event.preventDefault();
        pending = true;
        errorMessage = "";

        try {
            await login(repositories.authentication, userInfo);
            const redirectTarget = resolveRedirect(page.url.searchParams.get("redirect"));
            await goto(redirectTarget, {
                keepFocus: true,
                replaceState: true,
                noScroll: false,
            });
        } catch (error) {
            console.error(error);
            errorMessage = error instanceof Error ? error.message : "Login failed.";
        } finally {
            pending = false;
        }
    }

</script>

<div class="auth-page">
    <main class="auth-card">
        <h1>Sign In</h1>
        <p class="subtitle">Log in to continue.</p>

        {#if registeredMessage}
            <p class="success">Account created. Sign in to continue.</p>
        {/if}

        <form class="auth-form" onsubmit={handleLogin}>
            <input type="text" placeholder="Username" bind:value={userInfo.username} required />
            <input type="password" placeholder="Password" bind:value={userInfo.password} required />

            {#if errorMessage}
                <p class="error">{errorMessage}</p>
            {/if}

            <button type="submit" disabled={pending}>
                {pending ? "Signing in..." : "Sign In"}
            </button>
        </form>

        <a class="register-link" href={getRegisterHref()}>Create account</a>
    </main>
</div>

<style>
    .auth-page {
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 1rem;
        background: linear-gradient(160deg, #e2e8f0 0%, #f8fafc 55%, #e5e7eb 100%);
    }

    .auth-card {
        width: min(420px, 100%);
        border: 1px solid #d1d5db;
        border-radius: 12px;
        background: #fff;
        box-shadow: 0 10px 30px rgba(15, 23, 42, 0.08);
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.8rem;
    }

    h1 {
        margin: 0;
        font-size: 1.35rem;
    }

    .subtitle {
        margin: 0;
        color: #6b7280;
    }

    .auth-form {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .auth-form input {
        border: 1px solid #d1d5db;
        border-radius: 8px;
        padding: 0.6rem 0.7rem;
    }

    .auth-form button {
        border: 1px solid #111827;
        border-radius: 8px;
        background: #111827;
        color: #fff;
        padding: 0.55rem 0.75rem;
        cursor: pointer;
    }

    .auth-form button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .error {
        margin: 0;
        color: #b91c1c;
        font-weight: 600;
    }

    .register-link {
        color: #1d4ed8;
        text-decoration: none;
        width: fit-content;
    }

    .register-link:hover {
        text-decoration: underline;
    }

    .success {
        margin: 0;
        color: #166534;
        font-weight: 600;
    }
</style>
