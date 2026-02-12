<script lang="ts">
    export let variant: 'primary' | 'secondary' | 'danger' = 'primary';
    export let disabled = false;
</script>

<button
        class="btn btn-{variant}"
        {disabled}
        on:click>
    <slot />
</button>

<style>
    .btn {
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }
    .btn-primary { background: #ffbb00; color: white; }
    .btn-secondary { background: #0b0d1a; color: white; }
    .btn-danger { background: #ef4444; color: white; }
</style>