<script lang="ts">
  let {
    includeGeometry = $bindable(),
    includeMasks = $bindable(),
    includeOutput = $bindable(),
    bordered = false
  }: {
    includeGeometry: boolean;
    includeMasks: boolean;
    includeOutput: boolean;
    bordered?: boolean;
  } = $props();
</script>

<div
  class="flex flex-col gap-1 text-[11px] text-immich-dark-fg/60 {bordered
    ? 'border-t border-white/10 pt-3'
    : ''}"
>
  <span class="uppercase tracking-wider text-[10px] text-immich-dark-fg/40">Apply includes</span>
  <label class="flex items-center gap-2">
    <input type="checkbox" class="checkbox checkbox-xs" bind:checked={includeGeometry} />
    Geometry &amp; crop
  </label>
  <label class="flex items-center gap-2">
    <input type="checkbox" class="checkbox checkbox-xs" bind:checked={includeMasks} />
    Masks
  </label>
  <label class="flex items-center gap-2">
    <input type="checkbox" class="checkbox checkbox-xs" bind:checked={includeOutput} />
    Tonemap
  </label>
</div>
