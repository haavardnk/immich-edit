<script lang="ts">
  import SearchableSelect from '#lib/components/SearchableSelect.svelte';
  import { zoneLabel } from '#lib/metadata/photoTime.ts';

  interface Choice {
    id: string;
    label: string;
  }

  let {
    label,
    value,
    at,
    choices = [],
    class: className = 'w-60',
    onPick
  }: {
    label: string;
    value: string;
    at: number;
    choices?: readonly Choice[];
    class?: string;
    onPick: (id: string) => void;
  } = $props();

  const zones = Intl.supportedValuesOf('timeZone');
  const options = $derived([
    ...choices.map((choice) => choice.id),
    ...(zones.includes(value) || choices.some((choice) => choice.id === value) ? [] : [value]),
    ...zones
  ]);
  const named = $derived(new Map(choices.map((choice) => [choice.id, choice.label])));
</script>

<div class="{className} shrink-0 touch:w-full [&_.editor-compact-combobox_input]:h-8!">
  <SearchableSelect
    compact
    color="neutral"
    multiple={false}
    {options}
    selected={[value]}
    getId={(id) => id}
    getLabel={(id) => named.get(id) ?? zoneLabel(id, at)}
    getSearchText={(id) => named.get(id) ?? id.replaceAll('_', ' ')}
    {label}
    onSelectedChange={(next) => {
      const id = next[0];
      if (id) onPick(id);
    }}
  />
</div>
