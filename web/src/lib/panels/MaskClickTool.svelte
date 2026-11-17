<script lang="ts">
  import { segmented } from '#lib/components/segmented.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiCheck, mdiMinusCircleOutline, mdiPlusCircleOutline } from '@mdi/js';
  import { refineWith } from './maskTools';

  let { layerId, pending = false }: { layerId: string | null; pending?: boolean } = $props();

  const tool = $derived(editor.clickTool);
  const armed = $derived(tool.active && !tool.box && tool.layerId === layerId);
  const including = $derived(armed && !tool.negative);
  const excluding = $derived(armed && tool.negative);
  const touch = $derived(layout.mode === 'touch');
  const invertHint = $derived(touch ? '' : ` · ${keyLabel('Shift')}-click inverts one click`);
</script>

{#snippet toggle(negative: boolean, on: boolean)}
  <Button
    type="button"
    size="tiny"
    variant={on ? 'filled' : 'ghost'}
    color={on ? 'primary' : 'secondary'}
    leadingIcon={negative ? mdiMinusCircleOutline : mdiPlusCircleOutline}
    class="touch:h-11 touch:flex-1 touch:text-sm"
    title={negative
      ? `Tap an object to cut it out of the mask${invertHint}`
      : `Tap an object to add it to the mask${invertHint}`}
    aria-pressed={on}
    disabled={negative && pending}
    onclick={() => {
      if (!(pending && on)) refineWith(layerId, negative);
    }}
  >
    {negative ? 'Remove object' : 'Add object'}
  </Button>
{/snippet}

<div class="mt-1 flex h-7 items-center justify-between gap-1.5 touch:h-auto">
  <div class="{segmented.tool.group} flex text-[10px] touch:gap-2 touch:bg-transparent touch:p-0">
    {@render toggle(false, including)}
    {@render toggle(true, excluding)}
  </div>
  {#if armed && !touch}
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
{#if pending}
  <div class="text-[10px] text-dark/65 touch:text-[13px]">
    {touch ? 'Tap the photo to select an object.' : 'Click the photo to select an object.'}
  </div>
{/if}
{#if editor.maskGenerating}
  <div class="text-[10px] text-dark/65 touch:text-[13px]">Working…</div>
{/if}
