<script lang="ts">
  import { IconButton } from '@immich/ui';
  import { mdiAutoFix, mdiEyedropperVariant } from '@mdi/js';
  import { untrack } from 'svelte';
  import EditSlider from '#lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import { KELVIN, SHIFT_TEMPERATURE, SHIFT_TINT, TINT } from '#lib/editor/params/whiteBalance.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import {
    absoluteWhiteBalance,
    asShotWhiteBalance,
    relativeShift,
    relativeWhiteBalance,
    resolveTemperature,
    whiteBalanceIsAsShot,
    type Temperature
  } from '#lib/types/edits.ts';

  const wb = $derived(editor.edits.basic.white_balance);
  const asShot = $derived(editor.asShot);
  const pending = $derived(!asShot && !editor.asShotError);
  const white: Temperature | null = $derived.by(() => {
    if (asShot) return resolveTemperature(wb, asShot);
    return wb.mode === 'absolute' ? { kelvin: wb.kelvin, tint: wb.tint } : null;
  });
  const shift = $derived(asShot ? relativeShift(wb, asShot) : wb.mode === 'relative' ? wb : null);
  const kelvinMode = $derived(asShot ? asShot.raw : wb.mode === 'absolute');
  const modified = $derived(!whiteBalanceIsAsShot(wb));
  const dcpKey = $derived(
    `${editor.assetId}:${editor.initialised}:${JSON.stringify(editor.edits.color.dcp)}`
  );

  $effect(() => {
    void dcpKey;
    untrack(editor.loadAsShot);
  });

  function setWhite(next: Temperature): void {
    editor.edits.basic.white_balance = absoluteWhiteBalance(next, asShot);
    editor.onLive();
  }

  function setShift(temp: number, tint: number): void {
    editor.edits.basic.white_balance = relativeWhiteBalance(temp, tint);
    editor.onLive();
  }

  function reset(): void {
    editor.edits.basic.white_balance = asShotWhiteBalance();
    void editor.onCommit('Reset White Balance');
  }
</script>

<SectionHeader title="White Balance" section="white_balance" {modified} onReset={reset}>
  {#snippet actions()}
    <IconButton
      size="tiny"
      variant="ghost"
      color={editor.wbPicking ? 'primary' : 'secondary'}
      icon={mdiEyedropperVariant}
      title={hint('Pick a neutral area in the image', 'pickWhiteBalance')}
      aria-label="Pick white balance"
      aria-pressed={editor.wbPicking}
      disabled={editor.wbBusy || !editor.assetId}
      onclick={editor.toggleWbPicker}
    />
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      icon={mdiAutoFix}
      title="Auto white balance"
      aria-label="Auto white balance"
      disabled={editor.wbBusy || !editor.assetId}
      onclick={() => void editor.onAutoWhiteBalance()}
    />
  {/snippet}
</SectionHeader>
{#if kelvinMode && white}
  {@const kelvin = Math.round(white.kelvin)}
  {@const tint = Math.round(white.tint)}
  <EditSlider
    label="Temperature"
    commitAction="Temperature"
    value={kelvin}
    min={KELVIN.min}
    max={KELVIN.max}
    step={KELVIN.step}
    scale={KELVIN.scale}
    defaultValue={asShot?.kelvin ?? kelvin}
    disabled={pending}
    format={KELVIN.format}
    gradient={KELVIN.gradient}
    onLive={(v: number) => setWhite({ kelvin: v, tint })}
  />
  <EditSlider
    label="Tint"
    commitAction="Tint"
    value={tint}
    min={TINT.min}
    max={TINT.max}
    step={TINT.step}
    defaultValue={asShot?.tint ?? tint}
    disabled={pending}
    format={TINT.format}
    gradient={TINT.gradient}
    onLive={(v: number) => setWhite({ kelvin, tint: v })}
  />
{:else}
  {@const temp = Math.round(shift?.temp ?? 0)}
  {@const tint = Math.round(shift?.tint ?? 0)}
  <EditSlider
    label="Temperature"
    commitAction="Temperature"
    value={temp}
    min={SHIFT_TEMPERATURE.min}
    max={SHIFT_TEMPERATURE.max}
    step={SHIFT_TEMPERATURE.step}
    disabled={pending}
    format={SHIFT_TEMPERATURE.format}
    gradient={SHIFT_TEMPERATURE.gradient}
    onLive={(v: number) => setShift(v, tint)}
  />
  <EditSlider
    label="Tint"
    commitAction="Tint"
    value={tint}
    min={SHIFT_TINT.min}
    max={SHIFT_TINT.max}
    step={SHIFT_TINT.step}
    disabled={pending}
    format={SHIFT_TINT.format}
    gradient={SHIFT_TINT.gradient}
    onLive={(v: number) => setShift(temp, v)}
  />
{/if}
