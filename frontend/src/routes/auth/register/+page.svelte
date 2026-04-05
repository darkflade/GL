<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import type { UserInfo } from "$lib/domain/value-objects/auth";
    import { repositories } from "$lib/composition/repositories";
    import { register } from "$lib/application/use-cases/authentication";

    let userInfo: UserInfo = $state({
        username: "",
        password: "",
    });

    let pending = $state(false);
    let errorMessage = $state("");

    function resolveRedirect(target: string | null): string {
        if (!target || !target.startsWith("/") || target.startsWith("//")) {
            return "/feed";
        }
        return target;
    }

    function getLoginHref(): string {
        const redirect = resolveRedirect(page.url.searchParams.get("redirect"));
        return `/auth/login?redirect=${encodeURIComponent(redirect)}`;
    }

    async function handleRegister(event: SubmitEvent) {
        event.preventDefault();
        pending = true;
        errorMessage = "";

        try {
            await register(repositories.authentication, userInfo);
            const redirect = resolveRedirect(page.url.searchParams.get("redirect"));
            await goto(`/auth/login?redirect=${encodeURIComponent(redirect)}&registered=1`, {
                keepFocus: true,
                replaceState: true,
                noScroll: false,
            });
        } catch (error) {
            console.error(error);
            errorMessage = error instanceof Error ? error.message : "Registration failed.";
        } finally {
            pending = false;
        }

    }

</script>


<div class="auth-page">
    <main class="auth-card">
        <h1>Create Account</h1>
        <p class="subtitle">Register and continue to your target page.</p>

        <form class="auth-form" onsubmit={handleRegister}>
            <input type="text" placeholder="Username" bind:value={userInfo.username} required />
            <input type="password" placeholder="Password" bind:value={userInfo.password} required />

            {#if errorMessage}
                <p class="error">{errorMessage}</p>
            {/if}

            <button type="submit" disabled={pending}>
                {pending ? "Creating..." : "Create account"}
            </button>
        </form>

        <a class="login-link" href={getLoginHref()}>Already have an account? Sign in</a>
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

    .login-link {
        color: #1d4ed8;
        text-decoration: none;
        width: fit-content;
    }

    .login-link:hover {
        text-decoration: underline;
    }
</style>
