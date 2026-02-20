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
        
        // Refocus input after selection
        const input = document.querySelector('.search-input') as HTMLInputElement;
        if (input) input.focus();
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
            if (suggestions.length > 0 && activeIndex >= 0) {
                 event.preventDefault();
                 pickActiveSuggestion();
            } else {
                 submit();
            }
            return;
        }

        if (event.key === "Escape") {
            clearSuggestions();
        }
    }
</script>

<div class="search-container">
    <div class="search-input-wrapper glass-panel" class:focused={focused}>
        <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
            value={searchText}
            oninput={onInput}
            onkeydown={onKeydown}
            onfocus={() => (focused = true)}
            onblur={() => setTimeout(() => (focused = false), 200)}
            placeholder="Search tags... (e.g. nature, ~urban, -dark)"
            class="search-input"
        />
        {#if searchText}
            <button class="clear-button" onclick={() => { searchText = ''; submit(); }}>
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-4 h-4">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
            </button>
        {/if}
    </div>

    {#if focused && (loading || suggestions.length > 0)}
        <div class="suggestions-dropdown glass-panel animate-fade-in">
            {#if loading}
                <div class="suggestion-item muted">
                    <span class="loading-dots">Searching</span>
                </div>
            {:else}
                {#each suggestions as suggestion, index}
                    <button
                        type="button"
                        class="suggestion-item"
                        class:active={index === activeIndex}
                        onmousedown={(event) => {
                            event.preventDefault();
                            applySuggestion(suggestion.value);
                        }}
                    >
                        <span class="hash">#</span>
                        <span class="suggestion-text">{suggestion.value}</span>
                    </button>
                {/each}
            {/if}
        </div>
    {/if}
</div>

<style>
    .search-container {
        position: relative;
        width: 100%;
    }

    .search-input-wrapper {
        display: flex;
        align-items: center;
        padding: 0 var(--spacing-lg);
        height: 3.5rem;
        border-radius: var(--radius-full);
        transition: all 0.4s var(--ease-out);
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid var(--color-border);
    }

    .search-input-wrapper.focused {
        background: rgba(255, 255, 255, 0.06);
        border-color: var(--color-accent);
        box-shadow: 0 0 0 4px var(--color-accent-glow), var(--shadow-lg);
        transform: translateY(-1px);
    }

    .search-icon {
        width: 1.25rem;
        height: 1.25rem;
        color: var(--color-text-muted);
        margin-right: var(--spacing-sm);
        transition: color 0.3s ease;
    }

    .focused .search-icon {
        color: var(--color-accent);
    }

    .search-input {
        flex: 1;
        background: transparent;
        border: none;
        color: var(--color-text-primary);
        font-size: var(--text-base);
        font-weight: 500;
        outline: none;
        width: 100%;
    }

    .search-input::placeholder {
        color: var(--color-text-muted);
        opacity: 0.6;
    }

    .clear-button {
        background: rgba(255, 255, 255, 0.05);
        border: none;
        color: var(--color-text-muted);
        cursor: pointer;
        width: 24px;
        height: 24px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s ease;
    }

    .clear-button:hover {
        background: rgba(255, 255, 255, 0.1);
        color: var(--color-text-primary);
        transform: scale(1.1);
    }

    .suggestions-dropdown {
        position: absolute;
        top: calc(100% + 0.75rem);
        left: 0;
        right: 0;
        border-radius: var(--radius-xl);
        overflow: hidden;
        z-index: 50;
        padding: 8px;
        box-shadow: var(--shadow-xl);
        background: rgba(15, 15, 20, 0.95);
        backdrop-filter: blur(20px);
        border: 1px solid var(--color-border-bright);
    }

    .suggestion-item {
        width: 100%;
        display: flex;
        align-items: center;
        padding: 0.75rem 1rem;
        background: transparent;
        border: none;
        text-align: left;
        color: var(--color-text-secondary);
        cursor: pointer;
        transition: all 0.2s var(--ease-out);
        font-size: var(--text-sm);
        font-weight: 500;
        border-radius: var(--radius-md);
        margin-bottom: 2px;
    }

    .suggestion-item:last-child {
        margin-bottom: 0;
    }

    .suggestion-item:hover, .suggestion-item.active {
        background-color: rgba(255, 255, 255, 0.05);
        color: var(--color-text-primary);
        transform: translateX(4px);
    }

    .hash {
        color: var(--color-accent);
        margin-right: 0.75rem;
        font-weight: 800;
        opacity: 0.8;
    }

    .suggestion-item.muted {
        cursor: default;
        justify-content: center;
        padding: 1.5rem;
    }
    
    .loading-dots::after {
        content: '...';
        animation: dots 1.5s steps(4, end) infinite;
    }
    
    @keyframes dots {
        0%, 20% { content: ''; }
        40% { content: '.'; }
        60% { content: '..'; }
        80%, 100% { content: '...'; }
    }
</style>

