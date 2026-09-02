<script lang="ts">
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import { presets } from '$lib/stores/presets.svelte';

  let {
    selectedId = $bindable(),
    disabled = false,
    placeholder = 'Select a preset…'
  }: {
    selectedId: string | null;
    disabled?: boolean;
    placeholder?: string;
  } = $props();

  const options = $derived(presets.grouped.flatMap(({ items }) => items));

  function select(next: string[]): void {
    selectedId = next.at(-1) ?? null;
  }
</script>

<SearchableSelect
  compact
  multiple={false}
  color="neutral"
  {options}
  selected={selectedId ? [selectedId] : []}
  getId={(preset) => preset.id}
  getLabel={(preset) => preset.name}
  getGroup={(preset) => preset.group_name}
  {placeholder}
  {disabled}
  hideSelected={false}
  onSelectedChange={select}
/>
