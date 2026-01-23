<script lang="ts">
    import { Combobox } from "bits-ui";
    import { Check, ChevronsUpDown, X } from "lucide-svelte";
    import { cn } from "$lib/utils";
    import { searchTags } from "$lib/features/feed/api";
    import type {Tag} from "$lib/domain/models";

    let { onChange } = $props<{ onChange: (tags: Tag[]) => void }>();

    let selectedTags = $state<Tag[]>([]);

    let inputValue = $state("");
    let inputOpen = $state(false);

    let filteredTags = $state<Tag[]>([]);
    let timeout: ReturnType<typeof setTimeout> | undefined;

    async function debounceSearch() {
        if (inputValue.trim().length < 2) {
            filteredTags = [];
            return;
        }
        try {
            const res = await searchTags(inputValue);
            filteredTags = res?.filter(t => !selectedTags.some(s => s.id === t.id)) ?? [];
        } catch (err) {
            console.error(err);
            filteredTags = [];
        }
    }

    function handleInput(val: string) {
        inputValue = val;
        if (val.trim().length < 2) {
            filteredTags = [];
            return;
        }
        clearTimeout(timeout);
        timeout = setTimeout(async () => {
            const res = await searchTags(val);
            filteredTags = res ?? [];
        }, 300);
    }

    function removeTag(id: string) {
        selectedTags = selectedTags.filter(t => t.id !== id);
        onChange(selectedTags);
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Enter') {
            onChange(selectedTags);
        }
    }


    function selectTag(tag: Tag) {
        if (selectedTags.some(t => t.id === tag.id)) return;

        selectedTags = [...selectedTags, tag];
        inputValue = "";
        filteredTags = [];

        console.log("selectedTags called: ", selectedTags);
        onChange(selectedTags);
    }
</script>

<div class="w-full max-w-xs text-black">
    <Combobox.Root
            type="multiple"
            bind:open={inputOpen}
            onOpenChange={(next) => (inputOpen = next)}
            value={selectedTags.map(t => t.value)}
            onValueChange={(newValues) => {
                const lastSelectedValue = newValues[newValues.length - 1];
                const tagObj = filteredTags.find(t => t.value === lastSelectedValue);

                if (tagObj && !selectedTags.some(s => s.id === tagObj.id)) {
                    const newTags = [...selectedTags, tagObj];
                    selectedTags = newTags;
                    onChange(newTags);
                }
                inputValue = "";
            }}
    >
        <div class="relative">
            <div class="flex flex-wrap gap-1.5 pb-1.5">
                {#each selectedTags as tag (tag.id)}
                    <div class="inline-flex items-center gap-1 rounded bg-blue-100 px-2.5 py-1 text-sm text-blue-800">
                        {tag.value} <button type="button" onclick={() => removeTag(tag.id)}>
                        <X class="h-3.5 w-3.5" />
                    </button>
                    </div>
                {/each}
            </div>

            <div class="relative">
                <Combobox.Input
                        class={cn(
                        "flex h-10 w-full rounded-md border border-gray-300 bg-white px-3 py-2 pr-10 text-sm",
                        "placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    )}
                        placeholder={selectedTags.length ? "Add more..." : "Search tags..."}
                        oninput={(e) => handleInput(e.currentTarget.value)}
                        onkeydown={handleKeyDown}
                />
            </div>
        </div>

        <Combobox.Portal>
            <Combobox.Content
                    class="z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border bg-white py-1 shadow-lg ring-1 ring-black/5"
            >
                {#if filteredTags.length === 0 && inputValue.trim().length >= 2}
                    <div class="py-2 px-4 text-sm text-gray-500">No tags found</div>
                {/if}

                {#each filteredTags as tag (tag.id)}
                    <Combobox.Item
                            value={tag.value}
                            class="relative cursor-pointer select-none py-2 pl-3 pr-10 text-sm text-gray-900 data-[highlighted]:bg-blue-50"
                            onselect={() => selectTag(tag)}
                    >
                        <span class="block truncate">{tag.value}</span>
                        {#if selectedTags.some(t => t.id === tag.id)}
                             <span class="absolute inset-y-0 right-0 flex items-center pr-4 text-blue-600">
                                <Check class="h-4 w-4" />
                            </span>
                        {/if}
                    </Combobox.Item>
                {/each}
            </Combobox.Content>
        </Combobox.Portal>
    </Combobox.Root>
</div>