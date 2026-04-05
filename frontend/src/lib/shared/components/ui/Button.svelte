<script lang="ts">
    export let variant: 'primary' | 'secondary' | 'danger' = 'primary';
    export let disabled = false;
    export let href = "";
</script>
<a href={href}>
    <button
            class="btn btn-{variant}"
            {disabled}
            on:click>
        <slot />
    </button>
</a>

<style>
    .btn {
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }
    .btn-primary {
        background: #474747;
        color: white;
        border: .2rem inset white;
        border-radius: 8px
    }
    .btn-secondary { background: #0b0d1a; color: white; }
    .btn-danger { background: #ef4444; color: white; }
</style>