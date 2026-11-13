<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { MODES, modeVerb, patchShape } from '#lib/panels/maskTools.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { MaskLayer } from '#lib/types/edits.ts';
  import {
    mdiArrowDown,
    mdiArrowUp,
    mdiDeleteOutline,
    mdiFlagOutline,
    mdiInvertColors
  } from '@mdi/js';

  let {
    layer,
    componentId,
    label,
    onclose
  }: {
    layer: MaskLayer;
    componentId: string | null;
    label: string;
    onclose: () => void;
  } = $props();

  let shownId = $state<string | null>(null);

  $effect.pre(() => {
    if (componentId) shownId = componentId;
  });

  const index = $derived(layer.components.findIndex((comp) => comp.id === shownId));
  const comp = $derived(layer.components[index] ?? null);
</script>

<ActionSheet
  open={componentId !== null}
  onOpenChange={(next) => {
    if (!next) onclose();
  }}
  title={label || 'Shape'}
  caption
>
  {#if comp}
    {#if index > 0}
      {#each MODES as mode (mode.value)}
        <MenuRow
          icon={mode.icon}
          label={modeVerb(mode.value)}
          pressed={comp.mode === mode.value}
          onclick={() => patchShape(layer.id, comp.id, { mode: mode.value })}
        />
      {/each}
      <MenuRow
        icon={mdiFlagOutline}
        label="Make base"
        onclick={() => void editor.reorderMaskComponent(layer.id, comp.id, 0)}
      />
    {/if}
    <MenuRow
      icon={mdiInvertColors}
      label="Invert"
      pressed={comp.invert}
      onclick={() => patchShape(layer.id, comp.id, { invert: !comp.invert })}
    />
    <MenuRow
      icon={mdiArrowUp}
      label="Move up"
      disabled={index <= 0}
      onclick={() => void editor.reorderMaskComponent(layer.id, comp.id, index - 1)}
    />
    <MenuRow
      icon={mdiArrowDown}
      label="Move down"
      disabled={index >= layer.components.length - 1}
      onclick={() => void editor.reorderMaskComponent(layer.id, comp.id, index + 1)}
    />
    <MenuRow
      icon={mdiDeleteOutline}
      label="Delete"
      danger
      onclick={() => void editor.removeMaskComponent(layer.id, comp.id)}
    />
  {/if}
</ActionSheet>
