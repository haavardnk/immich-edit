<script lang="ts">
  import ToolbarButton from '$lib/components/ToolbarButton.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { ColorSpaceOpt } from '$lib/api/export';
  import { mdiMonitorEye } from '@mdi/js';

  let open = $state(false);
  let wideGamut = $state(true);

  $effect(() => {
    wideGamut = typeof window !== 'undefined' && window.matchMedia('(color-gamut: p3)').matches;
  });
</script>

<Popover
  {open}
  anchor="bottom"
  align="end"
  onClose={() => (open = false)}
  contentClass="p-2.5 w-56"
>
  {#snippet trigger()}
    <ToolbarButton
      path={mdiMonitorEye}
      size={18}
      title="Soft proof"
      active={editor.isProofing}
      onclick={() => (open = !open)}
    />
  {/snippet}
  {#snippet children()}
    <div class="flex flex-col gap-2.5">
      <div class="flex flex-col gap-1">
        <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Proof space</span>
        <select
          class="select bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2 min-h-0"
          value={editor.proofSpace}
          onchange={(e) =>
            editor.setProofSpace((e.currentTarget as HTMLSelectElement).value as ColorSpaceOpt)}
        >
          <option value="srgb">sRGB</option>
          <option value="displayp3" disabled={!wideGamut}>Display P3</option>
        </select>
        {#if !wideGamut}
          <span class="text-[10px] leading-tight text-immich-dark-fg/40">
            Display is not wide-gamut; P3 proof unavailable.
          </span>
        {/if}
      </div>
      <label class="flex items-center justify-between text-xs text-immich-dark-fg/80 select-none">
        <span>Show gamut warning</span>
        <input
          type="checkbox"
          class="accent-immich-dark-primary"
          checked={editor.gamutWarn}
          onchange={editor.toggleGamutWarn}
        />
      </label>
    </div>
  {/snippet}
</Popover>
