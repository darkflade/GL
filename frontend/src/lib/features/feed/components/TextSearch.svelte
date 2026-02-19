<script lang="ts">
    import type { Tag } from "$lib/domain/models/tag";
    import { searchTags } from "$lib/application/use-cases/search-tags";
    import { repositories } from "$lib/composition/repositories";
    import { escapeTagForSearch } from "$lib/utils/search";

    interface TagSuggestion {
        value: string;
    }

    let {
        onSearch,
        value = "",
    } = $props<{
        onSearch: (searchText: string) => void;
        value?: string;
    }>();

    let searchText = $state("");
    let suggestions = $state<TagSuggestion[]>([]);
    let loading = $state(false);
    let focused = $state(false);
    let activeIndex = $state(-1);
    let debounceTimer: ReturnType<typeof setTimeout> | undefined;

    $effect(() => {
        searchText = value;
    });

    function getActiveToken(input: string) {
        const endsWithSpace = /\s$/.test(input);
        if (endsWithSpace) {
            return { start: input.length, prefix: "", query: "" };
        }

        const start = input.lastIndexOf(" ") + 1;
        const token = input.slice(start);
        const prefix = token.startsWith("~") || token.startsWith("-") ? token[0] : "";
        const raw = prefix ? token.slice(1) : token;

        return {
            start,
            prefix,
            query: raw.replaceAll("_", " "),
        };
    }

    function setSuggestions(next: TagSuggestion[]) {
        suggestions = next;
        activeIndex = next.length > 0 ? 0 : -1;
    }

    function clearSuggestions() {
        setSuggestions([]);
    }

    function onInput(event: Event) {
        searchText = (event.currentTarget as HTMLInputElement).value;
        const { query } = getActiveToken(searchText);

        clearTimeout(debounceTimer);
        if (query.trim().length < 1) {
            clearSuggestions();
            return;
        }

        debounceTimer = setTimeout(async () => {
            loading = true;
            try {
                const tags = await searchTags(repositories.tags, query);
                const next = tags
                    .map((tag: Tag) => ({ value: escapeTagForSearch(tag.value) }))
                    .filter((tag) => tag.value.length > 0)
                    .slice(0, 8);
                setSuggestions(next);
            } catch (error) {
                console.error(error);
                clearSuggestions();
            } finally {
                loading = false;
            }
        }, 180);
    }

    function applySuggestion(tagName: string) {
        const { start, prefix } = getActiveToken(searchText);
        const before = searchText.slice(0, start);
        searchText = `${before}${prefix}${tagName} `;
        clearSuggestions();
    }

    function submit() {
        onSearch(searchText.trim());
        clearSuggestions();
    }

    function pickActiveSuggestion() {
        if (activeIndex < 0 || activeIndex >= suggestions.length) {
            return false;
        }

        applySuggestion(suggestions[activeIndex].value);
        return true;
    }

    function onKeydown(event: KeyboardEvent) {
        if (event.key === "ArrowDown" && suggestions.length > 0) {
            event.preventDefault();
            activeIndex = (activeIndex + 1 + suggestions.length) % suggestions.length;
            return;
        }

        if (event.key === "ArrowUp" && suggestions.length > 0) {
            event.preventDefault();
            activeIndex = (activeIndex - 1 + suggestions.length) % suggestions.length;
            return;
        }

        if (event.key === "Tab" && suggestions.length > 0) {
            event.preventDefault();
            pickActiveSuggestion();
            return;
        }

        if (event.key === "Enter") {
            event.preventDefault();
            if (!pickActiveSuggestion()) {
                submit();
            }
            return;
        }

        if (event.key === "Escape") {
            clearSuggestions();
        }
    }
</script>

<div class="text-search">
    <div class="text-search__controls">
        <input
            value={searchText}
            oninput={onInput}
            onkeydown={onKeydown}
            onfocus={() => (focused = true)}
            onblur={() => setTimeout(() => (focused = false), 120)}
            placeholder="tag_name ~tag_name -tag_name"
            class="text-search__input"
        />
        <button type="button" onclick={submit} class="text-search__button">Search</button>
    </div>

    {#if focused && (loading || suggestions.length > 0)}
        <div class="text-search__dropdown">
            {#if loading}
                <div class="text-search__item text-search__item--muted">Loading...</div>
            {:else}
                {#each suggestions as suggestion, index}
                    <button
                        type="button"
                        class={`text-search__item ${index === activeIndex ? "text-search__item--active" : ""}`}
                        onmousedown={(event) => {
                            event.preventDefault();
                            applySuggestion(suggestion.value);
                        }}
                    >
                        {suggestion.value}
                    </button>
                {/each}
            {/if}
        </div>
    {/if}
</div>

<style>
    .text-search {
        position: relative;
        width: min(860px, 100%);
    }

    .text-search__controls {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .text-search__input {
        flex: 1;
        min-width: 0;
        height: 2.5rem;
        border: 1px solid #d1d5db;
        border-radius: 0.5rem;
        padding: 0 0.75rem;
        font-size: 0.875rem;
    }

    .text-search__input:focus {
        outline: 2px solid #93c5fd;
        outline-offset: 1px;
    }

    .text-search__button {
        height: 2.5rem;
        border: 1px solid #1f2937;
        background: #111827;
        color: #fff;
        border-radius: 0.5rem;
        padding: 0 0.8rem;
        font-size: 0.875rem;
        cursor: pointer;
    }

    .text-search__dropdown {
        position: absolute;
        top: calc(100% + 0.25rem);
        width: 100%;
        border: 1px solid #e5e7eb;
        border-radius: 0.5rem;
        background: #fff;
        box-shadow: 0 8px 20px rgba(15, 23, 42, 0.12);
        z-index: 30;
        overflow: hidden;
    }

    .text-search__item {
        width: 100%;
        border: 0;
        background: #fff;
        text-align: left;
        padding: 0.55rem 0.75rem;
        font-size: 0.875rem;
        cursor: pointer;
    }

    .text-search__item:hover,
    .text-search__item--active {
        background: #eff6ff;
    }

    .text-search__item--muted {
        color: #6b7280;
        cursor: default;
    }
</style>
