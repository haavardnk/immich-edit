<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import {
    mdiArrowDown,
    mdiArrowUp,
    mdiContentCopy,
    mdiDeleteOutline,
    mdiEyeOffOutline,
    mdiEyeOutline,
    mdiInvertColors
  } from '@mdi/js';

  let { layerId, onclose }: { layerId: string | null; onclose: () => void } = $props();

  let shownId = $state<string | null>(null);

  $effect.pre(() => {
    if (layerId) shownId = layerId;
  });

  const layers = $derived(editor.edits.masks);
  const index = $derived(layers.findIndex((layer) => layer.id === shownId));
  const layer = $derived(layers[index] ?? null);
  const cap = $derived(editor.maskCapacityFor(null));

  async function duplicate(id: string): Promise<void> {
    await editor.duplicateMaskLayer(id);
    editor.setActiveMaskComponent(null);
  }
</script>

<ActionSheet
  open={layerId !== null}
  onOpenChange={(next) => {
    if (!next) onclose();
  }}
  title={layer?.name ?? 'Mask'}
  caption
>
  {#if layer}
    <MenuRow
      icon={mdiContentCopy}
      label="Duplicate"
      disabled={cap.layersFull || cap.totalFull}
      onclick={() => void duplicate(layer.id)}
    />
    <MenuRow
      icon={mdiInvertColors}
      label="Invert"
      pressed={layer.invert}
      disabled={layer.components.length === 0}
      onclick={() => void editor.toggleMaskLayerInvert(layer.id)}
    />
    <MenuRow
      icon={layer.enabled ? mdiEyeOffOutline : mdiEyeOutline}
      label={layer.enabled ? 'Turn off' : 'Turn on'}
      onclick={() => void editor.toggleMaskLayerEnabled(layer.id)}
    />
    <MenuRow
      icon={mdiArrowUp}
      label="Move earlier"
      disabled={index <= 0}
      onclick={() => void editor.reorderMaskLayer(layer.id, index - 1)}
    />
    <MenuRow
      icon={mdiArrowDown}
      label="Move later"
      disabled={index >= layers.length - 1}
      onclick={() => void editor.reorderMaskLayer(layer.id, index + 1)}
    />
    <MenuRow
      icon={mdiDeleteOutline}
      label="Delete"
      danger
      onclick={() => void editor.removeMaskLayer(layer.id)}
    />
  {/if}
</ActionSheet>
