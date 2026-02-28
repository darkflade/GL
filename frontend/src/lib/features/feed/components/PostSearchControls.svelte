<script lang="ts">
import TagSearch from "$lib/shared/components/layout/TagSearch.svelte";
import TextSearch from "$lib/features/feed/components/TextSearch.svelte";
import { queryFromTags, queryFromText } from "$lib/features/feed/search-query";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type { Tag } from "$lib/domain/models/tag";
import type { PaginationMode } from "$lib/features/feed/search-query";

let {
    value = "",
    paginationMode = "keyset",
    onQueryChange,
} = $props<{
    value?: string;
    paginationMode?: PaginationMode;
    onQueryChange: (query: SearchPostsQuery) => void;
}>();

function onTagsChange(tags: Tag[]) {
    onQueryChange(queryFromTags(tags, paginationMode));
}

function onTextSearch(searchText: string) {
    onQueryChange(queryFromText(searchText, paginationMode));
}
</script>

<div class="search-controls">
    <TextSearch value={value} onSearch={onTextSearch} />
    <TagSearch onChange={onTagsChange}/>
</div>

<style>
    .search-controls {
        flex: 1;
        min-width: 0;
        margin-left: 1rem;
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 0.75rem;
    }

    @media (max-width: 960px) {
        .search-controls {
            margin-left: 0.5rem;
            gap: 0.5rem;
        }
    }
</style>
