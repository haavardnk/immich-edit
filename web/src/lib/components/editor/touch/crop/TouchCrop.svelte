<script lang="ts">
  import { useGeometrySession } from '#lib/panels/geometry/geometrySession.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { touchUi, type TrayDetent } from '#lib/stores/touchUi.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import CropDial from './CropDial.svelte';
  import CropMoreSheet from './CropMoreSheet.svelte';
  import CropTools from './CropTools.svelte';
  import RatioKeypad from './RatioKeypad.svelte';
  import RatioSheet from './RatioSheet.svelte';
  import UprightTray from './UprightTray.svelte';

  let { stacked = false }: { stacked?: boolean } = $props();

  useGeometrySession();

  let ratioOpen = $state(false);
  let moreOpen = $state(false);
  let custom = $state(false);
  let before: TrayDetent = 'compact';

  function openCustom(): void {
    before = touchUi.detent;
    custom = true;
    touchUi.setDetent('expanded');
  }

  function closeCustom(): void {
    custom = false;
    touchUi.setDetent(before);
  }
</script>

{#if editor.geometrySession}
  {@const sess = editor.geometrySession}
  <div class="flex min-h-0 flex-1 flex-col overflow-y-auto overscroll-contain scrollbar-hidden">
    {#if custom}
      <RatioKeypad onclose={closeCustom} />
    {:else}
      {#if ui.geometryTool === 'upright'}
        <UprightTray {sess} />
      {:else}
        <CropDial {sess} fill={!stacked} />
      {/if}
      <div class={['shrink-0', !stacked && 'mt-auto']}>
        <CropTools onratio={() => (ratioOpen = true)} onmore={() => (moreOpen = true)} />
      </div>
    {/if}
  </div>
{/if}
<RatioSheet open={ratioOpen} onOpenChange={(next) => (ratioOpen = next)} oncustom={openCustom} />
<CropMoreSheet open={moreOpen} onOpenChange={(next) => (moreOpen = next)} />
