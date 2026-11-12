<script lang="ts">
  import type { CropRatio } from '#lib/api/preferences.ts';
  import TextInput from '#lib/components/TextInput.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import {
    isPresetRatio,
    longSideFirst,
    orientedAspect,
    parseRatioSide,
    PRESET_RATIOS,
    ratioKey,
    ratioLabel
  } from '#lib/panels/geometry/aspects.ts';
  import { cropReadout } from '#lib/panels/geometry/cropReadout.ts';
  import {
    draftPortrait,
    draftRatio,
    isPicked,
    isSavedRatio
  } from '#lib/panels/geometry/ratioActions.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import {
    mdiBackspaceOutline,
    mdiBookmarkOutline,
    mdiBookmarkRemoveOutline,
    mdiCropLandscape,
    mdiSwapHorizontal
  } from '@mdi/js';
  import { untrack } from 'svelte';

  type Side = 'width' | 'height';

  const MAX_DIGITS = 4;
  const DIGITS = ['1', '2', '3', '4', '5', '6', '7', '8', '9'] as const;

  let { onclose }: { onclose: () => void } = $props();

  const start = untrack(() => draftRatio());
  let sides = $state<Record<Side, string>>({
    width: start ? String(start.num) : '',
    height: start ? String(start.den) : ''
  });
  let side = $state<Side>('width');

  const readout = $derived(cropReadout());
  const typed = $derived.by((): CropRatio | null => {
    const num = parseRatioSide(sides.width);
    const den = parseRatioSide(sides.height);
    return num !== null && den !== null ? { num, den } : null;
  });
  const savable = $derived(typed && !isPresetRatio(typed) ? longSideFirst(typed) : null);
  const saved = $derived(savable !== null && isSavedRatio(savable));
  const savedRatios = $derived(preferences.savedRatios);

  function keep(e: PointerEvent): void {
    e.preventDefault();
  }

  function press(digit: string): void {
    const current = sides[side];
    if (current.length >= MAX_DIGITS) return;
    sides[side] = current === '0' ? digit : current + digit;
  }

  function erase(): void {
    sides[side] = sides[side].slice(0, -1);
  }

  function swap(): void {
    sides = { width: sides.height, height: sides.width };
  }

  function clear(): void {
    sides = { width: '', height: '' };
    side = 'width';
  }

  function apply(): void {
    if (typed) void editor.setGeometryAspect({ kind: 'ratio', ...typed });
  }

  function done(): void {
    apply();
    onclose();
  }

  function toggleSave(): void {
    if (!savable) return;
    if (saved) {
      void preferences.removeCropRatio(savable);
      return;
    }
    apply();
    void preferences.saveCropRatio(savable);
  }

  function pick(ratio: CropRatio): void {
    const aspect = orientedAspect(ratio, draftPortrait());
    if (aspect.kind !== 'ratio') return;
    sides = { width: String(aspect.num), height: String(aspect.den) };
    void editor.setGeometryAspect(aspect);
  }

  function onField(which: Side, e: Event): void {
    const value = (e.currentTarget as HTMLInputElement).value.replace(/\D/g, '');
    sides[which] = value.slice(0, MAX_DIGITS);
  }

  function onFieldKey(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      done();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onclose();
    } else if (e.key === ':') {
      e.preventDefault();
      side = 'height';
    }
  }
</script>

{#snippet chip(ratio: CropRatio)}
  <Button
    size="small"
    variant="ghost"
    color="secondary"
    shape="round"
    class="shrink-0 px-3.5 text-sm font-normal {isPicked(ratio)
      ? 'bg-primary/22 text-light-950'
      : 'bg-ghost text-dark'}"
    aria-pressed={isPicked(ratio)}
    onclick={() => pick(ratio)}>{ratioLabel(ratio)}</Button
  >
{/snippet}

{#snippet field(which: Side, label: string)}
  <div
    class="relative min-w-0 flex-1 rounded-[10px] {side === which
      ? 'outline-2 outline-primary'
      : ''}"
  >
    <span
      class="pointer-events-none absolute top-1/2 left-3 z-10 -translate-y-1/2 text-[13px] text-dark/65"
      aria-hidden="true">{label}</span
    >
    <TextInput
      inputmode="none"
      autocomplete="off"
      class="h-11 w-full rounded-[10px] bg-light-100! [&_input]:ps-16 [&_input]:pe-3 [&_input]:text-right [&_input]:font-mono [&_input]:text-base [&_input]:tabular-nums"
      aria-label="Ratio {label.toLowerCase()}"
      value={sides[which]}
      onfocus={() => (side = which)}
      oninput={(e) => onField(which, e)}
      onkeydown={onFieldKey}
    />
  </div>
{/snippet}

{#snippet key(label: string, onclick: () => void, extra = '')}
  <Button
    size="large"
    variant="ghost"
    color="secondary"
    class="h-11 rounded-lg text-xl font-normal {extra}"
    onpointerdown={keep}
    {onclick}>{label}</Button
  >
{/snippet}

<section
  aria-label="Custom ratio"
  class="flex shrink-0 flex-col gap-2 px-3 pt-1 pb-(--tab-bar-pad)"
>
  <div class="flex items-center gap-3 px-1">
    <Icon icon={mdiCropLandscape} size="22px" class="shrink-0 text-primary" aria-hidden="true" />
    <div class="min-w-0 flex-1">
      <p class="text-[15px] font-semibold text-light-950">Aspect ratio</p>
      {#if readout}
        <p class="truncate text-[12px] text-dark/65 tabular-nums">
          {readout.label} at {readout.lead}
        </p>
      {/if}
    </div>
    <Button
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      class="bg-ghost"
      leadingIcon={saved ? mdiBookmarkRemoveOutline : mdiBookmarkOutline}
      disabled={!savable}
      onclick={toggleSave}>{saved ? 'Remove ratio' : 'Save ratio'}</Button
    >
  </div>
  <div class="-mx-3 flex gap-1.5 overflow-x-auto px-3 scrollbar-hidden" aria-label="Presets">
    {#each PRESET_RATIOS as ratio (ratioKey(ratio))}
      {@render chip(ratio)}
    {/each}
  </div>
  {#if savedRatios.length}
    <div class="-mx-3 flex gap-1.5 overflow-x-auto px-3 scrollbar-hidden" aria-label="Saved">
      {#each savedRatios as ratio (ratioKey(ratio))}
        {@render chip(ratio)}
      {/each}
    </div>
  {/if}
  <div class="flex items-center gap-2">
    {@render field('width', 'Width')}
    <IconButton
      size="large"
      variant="ghost"
      color="secondary"
      class="size-11 shrink-0 rounded-[10px] bg-light-200"
      icon={mdiSwapHorizontal}
      title="Swap width and height"
      aria-label="Swap width and height"
      onpointerdown={keep}
      onclick={swap}
    />
    {@render field('height', 'Height')}
  </div>
  <div class="grid grid-cols-4 gap-1.5">
    {#each DIGITS as digit, index (digit)}
      {@render key(digit, () => press(digit), 'bg-light-200 text-light-950 hover:bg-light-300')}
      {#if index === 2}
        <IconButton
          size="large"
          variant="ghost"
          color="secondary"
          class="h-11 w-full rounded-lg"
          icon={mdiBackspaceOutline}
          title="Delete"
          aria-label="Delete"
          onpointerdown={keep}
          onclick={erase}
        />
      {:else if index === 5}
        {@render key(':', () => (side = 'height'))}
      {:else if index === 8}
        <span
          class="flex items-center justify-center text-center text-[11px] text-dark/65 tabular-nums"
          >1 … 9999</span
        >
      {/if}
    {/each}
    {@render key('Clear', clear, 'text-sm font-medium')}
    {@render key('0', () => press('0'), 'bg-light-200 text-light-950 hover:bg-light-300')}
    <Button
      size="large"
      color="primary"
      class="col-span-2 h-11 rounded-lg text-sm font-medium"
      onpointerdown={keep}
      disabled={!typed}
      onclick={done}>Done</Button
    >
  </div>
</section>
