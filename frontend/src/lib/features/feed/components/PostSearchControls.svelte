<script lang="ts">
    import TextSearch from "$lib/features/feed/components/TextSearch.svelte";
    import { queryFromTags, queryFromText } from "$lib/features/feed/search-query";
    import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
    import type { Tag } from "$lib/domain/models/tag";

    let {
        value = "",
        onQueryChange,
    } = $props<{
        value?: string;
        onQueryChange: (query: SearchPostsQuery) => void;
    }>();

    function onTextSearch(searchText: string) {
        onQueryChange(queryFromText(searchText));
    }
</script>

<div class="search-controls-wrapper">
    <TextSearch value={value} onSearch={onTextSearch} />
</div>

<style>
    .search-controls-wrapper {
        width: 100%;
        display: flex;
        justify-content: flex-end;
    }
</style>
