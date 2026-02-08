<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { goto } from '$app/navigation';
	import {onMount} from "svelte";

	export const prerender = true;
	export const ssr = false;

	let { children } = $props();

	onMount(() => {
		//TODO localstorage contains default page
		goto("/feed", {replaceState: false})
	})
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
