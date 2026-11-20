<script lang="ts">
import TagSearch from "$lib/shared/components/layout/TagSearch.svelte";
import TextSearch from "$lib/features/feed/components/TextSearch.svelte";
import { queryFromTags, queryFromText } from "$lib/features/feed/search-query";
import type { SearchPostsQuery } from "$lib/domain/value-objects/search";
import type { Tag } from "$lib/domain/models/tag";
import type { PaginationMode } from "$lib/features/feed/search-query";

let {
    value = "",
    sort = "relevance",
    paginationMode = "keyset",
    onQueryChange,
} = $props<{
    value?: string;
    sort?: SearchPostsQuery["sort"];
    paginationMode?: PaginationMode;
    onQueryChange: (query: SearchPostsQuery) => void;
}>();

function onTagsChange(tags: Tag[]) {
    onQueryChange({ ...queryFromTags(tags, paginationMode), sort });
}

function onTextSearch(searchText: string) {
    onQueryChange({ ...queryFromText(searchText, paginationMode), sort });
}
</script>

<div class="search-controls">
    <label>Order <select aria-label="Sort by UUID" value={sort} onchange={(event) => onQueryChange({ ...queryFromText(value, paginationMode), sort: event.currentTarget.value as SearchPostsQuery["sort"] })}><option value="relevance">Relevance</option><option value="uuid_desc">UUID ↓</option><option value="uuid_asc">UUID ↑</option></select></label>
    <TextSearch value={value} onSearch={onTextSearch} />
    <TagSearch onChange={onTagsChange}/>
</div>

<style>
    label { display: flex; align-items: center; gap: .35rem; white-space: nowrap; }
    select { border: 1px solid #d1d5db; border-radius: 6px; padding: .35rem; }
    .search-controls {
        flex-wrap: wrap;
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
