<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';

  let {
    includeGeometry = $bindable(),
    includeMasks = $bindable(),
    bordered = false
  }: {
    includeGeometry: boolean;
    includeMasks: boolean;
    bordered?: boolean;
  } = $props();
</script>

<div
  class="flex flex-col gap-1 text-[11px] text-dark/65 {bordered
    ? 'border-t border-dark/10 pt-3'
    : ''}"
>
  <span class="uppercase tracking-wider text-[10px] text-dark/65">Apply includes</span>
  <CheckboxRow
    label="Geometry & crop"
    checked={includeGeometry}
    onChange={(v) => (includeGeometry = v)}
  />
  <CheckboxRow label="Masks" checked={includeMasks} onChange={(v) => (includeMasks = v)} />
</div>
