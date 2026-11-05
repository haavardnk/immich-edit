<script lang="ts">
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import ResetButton from '#lib/components/editor/controls/ResetButton.svelte';
  import MaskedEditSlider from './MaskedEditSlider.svelte';
  import { MASK_AMOUNT, MASK_PARAM_ROWS } from '#lib/editor/params/masks.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  let { layerId }: { layerId: string } = $props();

  const layer = $derived(editor.edits.masks.find((l) => l.id === layerId) ?? null);
  const amountValue = $derived(layer?.amount ?? MASK_AMOUNT.defaultValue);
  const isDefault = $derived(
    amountValue === MASK_AMOUNT.defaultValue &&
      Object.values(layer?.edits ?? {}).every((v) => v === 0)
  );

  function onAmountLive(v: number): void {
    editor.setMaskLayerAmount(layerId, v);
  }

  function commit(): void {
    void editor.commitMasks();
  }

  function reset(): void {
    void editor.resetMaskLayerEdits(layerId);
  }
</script>

<div class="mt-2 flex flex-col gap-1.5 border-t border-dark/10 pt-2">
  <div class="flex items-center gap-1 px-1">
    <div class="flex-1 min-w-0 text-[10px] uppercase tracking-wider text-dark/65 truncate">
      Adjustments · {layer?.name ?? ''}
    </div>
    <ResetButton
      title="Reset every adjustment on this mask"
      label="Reset mask adjustments"
      disabled={isDefault}
      onclick={reset}
    />
  </div>
  <SliderRow
    label="Amount"
    value={amountValue}
    range={MASK_AMOUNT}
    onLive={onAmountLive}
    onCommit={commit}
    onDragStart={editor.holdMaskOverlay}
    onDragEnd={editor.releaseMaskOverlay}
  />
  {#each MASK_PARAM_ROWS as row, index (index)}
    <div class="border-t border-dark/5"></div>
    {#each row as param (param.key)}
      <MaskedEditSlider {layerId} {param} />
    {/each}
  {/each}
</div>
