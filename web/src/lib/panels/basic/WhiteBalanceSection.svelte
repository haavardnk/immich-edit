<script lang="ts">
  import { IconButton } from '@immich/ui';
  import { mdiAutoFix, mdiEyedropperVariant } from '@mdi/js';
  import { untrack } from 'svelte';
  import EditSlider from '#lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import { commitSectionReset } from '#lib/components/editor/controls/paramReset.ts';
  import { settingFilter } from '#lib/components/editor/controls/settingFilter.ts';
  import {
    KELVIN,
    SHIFT_TEMPERATURE,
    SHIFT_TINT,
    TINT,
    WHITE_BALANCE
  } from '#lib/editor/params/whiteBalance.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import {
    absoluteWhiteBalance,
    relativeShift,
    relativeWhiteBalance,
    resolveTemperature,
    whiteBalanceIsAsShot,
    type Temperature
  } from '#lib/types/edits.ts';

  const filter = settingFilter();
  const shown = $derived(!filter || filter.shows('section:white_balance'));
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
</script>

{#if shown}
  <SectionHeader
    title="White Balance"
    section="white_balance"
    {modified}
    onReset={() => commitSectionReset(WHITE_BALANCE)}
  >
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
      range={KELVIN}
      defaultValue={asShot?.kelvin ?? kelvin}
      disabled={pending}
      onLive={(v: number) => setWhite({ kelvin: v, tint })}
    />
    <EditSlider
      label="Tint"
      commitAction="Tint"
      value={tint}
      range={TINT}
      defaultValue={asShot?.tint ?? tint}
      disabled={pending}
      onLive={(v: number) => setWhite({ kelvin, tint: v })}
    />
  {:else}
    {@const temp = Math.round(shift?.temp ?? 0)}
    {@const tint = Math.round(shift?.tint ?? 0)}
    <EditSlider
      label="Temperature"
      commitAction="Temperature"
      value={temp}
      range={SHIFT_TEMPERATURE}
      disabled={pending}
      onLive={(v: number) => setShift(v, tint)}
    />
    <EditSlider
      label="Tint"
      commitAction="Tint"
      value={tint}
      range={SHIFT_TINT}
      disabled={pending}
      onLive={(v: number) => setShift(temp, v)}
    />
  {/if}
{/if}
