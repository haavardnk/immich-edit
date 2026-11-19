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
    size={touch ? 'medium' : 'tiny'}
    variant={on ? 'filled' : 'ghost'}
    color={on ? 'primary' : 'secondary'}
    leadingIcon={negative ? mdiMinusCircleOutline : mdiPlusCircleOutline}
    class={touch
      ? `h-11 min-w-0 gap-1.5 rounded-xl px-2 text-sm font-normal ${on ? '' : 'bg-ghost'}`
      : ''}
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
  <div class={touch ? 'grid w-full grid-cols-2 gap-2' : `${segmented.tool.group} flex text-micro`}>
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
  <div class="text-micro text-dark/65 touch:text-label">
    {touch ? 'Tap the photo to select an object.' : 'Click the photo to select an object.'}
  </div>
{/if}
{#if editor.maskGenerating}
  <div class="text-micro text-dark/65 touch:text-label">Working…</div>
{/if}
