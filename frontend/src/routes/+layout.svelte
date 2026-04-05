<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}

<style>
	:global(header){
		margin-inline: 1rem;
	}
	:global(main) {
		padding: 1rem;
	}
	:global(body) {
		background-color: #ececec;
	}
</style>
