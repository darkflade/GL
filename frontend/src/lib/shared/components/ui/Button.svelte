<script lang="ts">
    import { type Snippet } from 'svelte';

    type Variant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'outline';
    type Size = 'sm' | 'md' | 'lg' | 'icon';

    let {
        variant = 'primary',
        size = 'md',
        disabled = false,
        href = "",
        onclick,
        children
    } = $props<{
        variant?: Variant;
        size?: Size;
        disabled?: boolean;
        href?: string;
        onclick?: (e: MouseEvent) => void;
        children?: Snippet;
    }>();

    const baseClass = "btn";
    const variantClass = `btn-${variant}`;
    const sizeClass = `btn-${size}`;
</script>

{#if href}
    <a {href} class="{baseClass} {variantClass} {sizeClass}" aria-disabled={disabled}>
        {@render children?.()}
    </a>
{:else}
    <button
        class="{baseClass} {variantClass} {sizeClass}"
        {disabled}
        onclick={onclick}
    >
        {@render children?.()}
    </button>
{/if}

<style>
    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border-radius: var(--radius-md);
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s var(--ease-out);
        text-decoration: none;
        border: 1px solid transparent;
        gap: 0.5rem;
        font-size: var(--text-sm);
        white-space: nowrap;
        position: relative;
        overflow: hidden;
    }

    .btn:active {
        transform: scale(0.96);
    }

    .btn:disabled, .btn[aria-disabled="true"] {
        opacity: 0.4;
        pointer-events: none;
        filter: grayscale(1);
    }

    /* Sizes */
    .btn-sm {
        height: 2.25rem;
        padding: 0 0.875rem;
        font-size: var(--text-xs);
    }
    .btn-md {
        height: 2.75rem;
        padding: 0 1.25rem;
    }
    .btn-lg {
        height: 3.5rem;
        padding: 0 1.75rem;
        font-size: var(--text-base);
        border-radius: var(--radius-lg);
    }
    .btn-icon {
        height: 2.75rem;
        width: 2.75rem;
        padding: 0;
    }

    /* Variants */
    .btn-primary {
        background: linear-gradient(135deg, var(--color-accent) 0%, #4f46e5 100%);
        color: white;
        border: 1px solid rgba(255, 255, 255, 0.1);
        box-shadow: 0 4px 12px rgba(99, 102, 241, 0.2);
    }
    .btn-primary:hover {
        transform: translateY(-2px);
        box-shadow: 0 8px 20px rgba(99, 102, 241, 0.3);
        background: linear-gradient(135deg, var(--color-accent-vibrant) 0%, var(--color-accent) 100%);
    }

    .btn-secondary {
        background-color: var(--color-surface-raised);
        color: var(--color-text-primary);
        border: 1px solid var(--color-border-bright);
    }
    .btn-secondary:hover {
        background-color: var(--color-surface-hover);
        border-color: var(--color-text-muted);
        transform: translateY(-1px);
    }

    .btn-outline {
        background-color: transparent;
        border: 1px solid var(--color-border-bright);
        color: var(--color-text-secondary);
    }
    .btn-outline:hover {
        border-color: var(--color-text-primary);
        color: var(--color-text-primary);
        background-color: rgba(255, 255, 255, 0.05);
    }

    .btn-ghost {
        background-color: transparent;
        color: var(--color-text-secondary);
    }
    .btn-ghost:hover {
        background-color: var(--color-surface-hover);
        color: var(--color-text-primary);
    }

    .btn-danger {
        background-color: rgba(255, 77, 77, 0.1);
        color: var(--color-danger);
        border: 1px solid rgba(255, 77, 77, 0.2);
    }
    .btn-danger:hover {
        background-color: var(--color-danger);
        color: white;
        transform: translateY(-1px);
    }
</style>
