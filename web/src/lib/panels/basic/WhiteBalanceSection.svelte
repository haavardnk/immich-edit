<script lang="ts">
  import { IconButton } from '@immich/ui';
  import { mdiAutoFix, mdiEyedropperVariant } from '@mdi/js';
  import { untrack } from 'svelte';
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import { miredScale } from '$lib/components/editor/controls/sliderScale';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    KELVIN_MAX,
    KELVIN_MIN,
    RELATIVE_LIMIT,
    TINT_LIMIT,
    absoluteWhiteBalance,
    asShotWhiteBalance,
    relativeShift,
    relativeWhiteBalance,
    resolveTemperature,
    whiteBalanceIsAsShot,
    type Temperature
  } from '$lib/types/edits';

  const kelvinScale = miredScale(KELVIN_MIN, KELVIN_MAX);

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
      size="small"
      variant="ghost"
      color={editor.wbPicking ? 'primary' : 'secondary'}
      class="size-6 rounded p-0 {editor.wbPicking
        ? 'bg-white/10'
        : 'text-dark/55 hover:bg-hairline hover:text-dark'}"
      icon={mdiEyedropperVariant}
      title="Pick a neutral area in the image"
      aria-label="Pick white balance"
      aria-pressed={editor.wbPicking}
      disabled={editor.wbBusy || !editor.assetId}
      onclick={editor.toggleWbPicker}
    />
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="size-6 rounded p-0 text-dark/55 hover:bg-hairline hover:text-dark"
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
    min={KELVIN_MIN}
    max={KELVIN_MAX}
    scale={kelvinScale}
    defaultValue={asShot?.kelvin ?? kelvin}
    disabled={pending}
    format={(v: number) => `${v.toFixed(0)} K`}
    gradient="var(--gradient-temperature)"
    onLive={(v: number) => setWhite({ kelvin: v, tint })}
  />
  <EditSlider
    label="Tint"
    commitAction="Tint"
    value={tint}
    min={-TINT_LIMIT}
    max={TINT_LIMIT}
    defaultValue={asShot?.tint ?? tint}
    disabled={pending}
    gradient="var(--gradient-tint)"
    onLive={(v: number) => setWhite({ kelvin, tint: v })}
  />
{:else}
  {@const temp = Math.round(shift?.temp ?? 0)}
  {@const tint = Math.round(shift?.tint ?? 0)}
  <EditSlider
    label="Temperature"
    commitAction="Temperature"
    value={temp}
    min={-RELATIVE_LIMIT}
    max={RELATIVE_LIMIT}
    disabled={pending}
    gradient="var(--gradient-temperature)"
    onLive={(v: number) => setShift(v, tint)}
  />
  <EditSlider
    label="Tint"
    commitAction="Tint"
    value={tint}
    min={-RELATIVE_LIMIT}
    max={RELATIVE_LIMIT}
    disabled={pending}
    gradient="var(--gradient-tint)"
    onLive={(v: number) => setShift(temp, v)}
  />
{/if}
