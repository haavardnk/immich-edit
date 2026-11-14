<script lang="ts">
  import type { ClickPoint } from '#lib/api/masks.ts';
  import { segmented } from '#lib/components/segmented.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiCheck, mdiMinusCircleOutline, mdiPlusCircleOutline } from '@mdi/js';
  import { refineWith } from './maskTools';

  let {
    layerId,
    points = [],
    pending = false
  }: { layerId: string | null; points?: ClickPoint[]; pending?: boolean } = $props();

  const tool = $derived(editor.clickTool);
  const armed = $derived(tool.active && !tool.box && tool.layerId === layerId);
  const including = $derived(armed && !tool.negative);
  const excluding = $derived(armed && tool.negative);
  const included = $derived(points.filter((p) => p.positive).length);
  const help = $derived.by(() => {
    if (pending)
      return layout.mode === 'touch'
        ? 'Tap the photo to select an area.'
        : 'Click the photo to select an area.';
    const lead =
      layout.mode === 'touch'
        ? 'Tap a dot to delete it.'
        : `${keyLabel('Shift')}-click or right-click inverts one click. Click a dot to delete it.`;
    if (points.length === 0) return lead;
    return `${lead} ${included} included, ${points.length - included} excluded.`;
  });
</script>

<div class="mt-1 flex h-7 items-center justify-between gap-1.5">
  <div class="{segmented.tool.group} flex text-[10px]">
    <Button
      type="button"
      size="tiny"
      variant={including ? 'filled' : 'ghost'}
      color={including ? 'primary' : 'secondary'}
      leadingIcon={mdiPlusCircleOutline}
      title="Clicks add that area to the mask"
      aria-pressed={including}
      onclick={() => refineWith(layerId, false)}
    >
      Include
    </Button>
    <Button
      type="button"
      size="tiny"
      variant={excluding ? 'filled' : 'ghost'}
      color={excluding ? 'primary' : 'secondary'}
      leadingIcon={mdiMinusCircleOutline}
      title={pending ? 'Include an area first' : 'Clicks cut that area out of the mask'}
      aria-pressed={excluding}
      disabled={pending}
      onclick={() => refineWith(layerId, true)}
    >
      Exclude
    </Button>
  </div>
  {#if armed}
    <Button
      type="button"
      size="tiny"
      color="primary"
      leadingIcon={mdiCheck}
      onclick={editor.stopClickTool}
    >
      Done
    </Button>
  {/if}
</div>
<div class="text-[10px] text-dark/65">{help}</div>
{#if editor.maskGenerating}
  <div class="text-[10px] text-dark/65">Working…</div>
{/if}
