<script lang="ts">
  import { hint } from '$lib/keybinds';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui, CROP_GRIDS } from '$lib/stores/ui.svelte';
  import type { GeometrySession } from '$lib/stores/editor/geometry.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { croppedOutputSize } from '$lib/utils/geom';
  import {
    aspectForKey,
    aspectOptions,
    CUSTOM_KEY,
    isPortraitAspect,
    parseRatioSide,
    selectedAspectKey
  } from './aspects';
  import { Field, IconButton, Select } from '@immich/ui';
  import { mdiCropLandscape, mdiCropPortrait, mdiGrid } from '@mdi/js';

  let { sess }: { sess: GeometrySession } = $props();

  let customPicked = $state(false);
  let customDraft = $state({ num: '', den: '' });

  const ratioInputClass =
    'ring-0 focus-within:ring-1 focus-within:ring-primary [&_input]:h-7 [&_input]:px-2';

  const aspect = $derived(sess.draftAspect);
  const portrait = $derived(isPortraitAspect(aspect));
  const selectedKey = $derived(customPicked ? CUSTOM_KEY : selectedAspectKey(aspect));
  const customValues = $derived(
    aspect.kind === 'ratio' ? { num: String(aspect.num), den: String(aspect.den) } : customDraft
  );
  const orientationAvailable = $derived(aspect.kind === 'ratio' && aspect.num !== aspect.den);
  const cropModified = $derived(sess.userEditedCrop || aspect.kind !== 'original');
  const gridLabel = $derived(CROP_GRIDS.find((g) => g.id === ui.cropGrid)?.label ?? '');
  const outputSize = $derived(
    editor.meta
      ? croppedOutputSize(
          {
            ...editor.edits.geometry,
            rotate: sess.draftRotate,
            rotate_angle: sess.draftAngle,
            crop: sess.draftCrop
          },
          editor.meta.source_w,
          editor.meta.source_h
        )
      : null
  );

  function onAspectChange(key: string): void {
    customPicked = key === CUSTOM_KEY;
    const next = aspectForKey(key, portrait);
    if (next) editor.updateGeometryDraftAspect(next);
  }

  function setCustomSide(side: 'num' | 'den', raw: string): void {
    customDraft = { ...customValues, [side]: raw };
    const num = parseRatioSide(customDraft.num);
    const den = parseRatioSide(customDraft.den);
    if (num !== null && den !== null) editor.updateGeometryDraftAspect({ kind: 'ratio', num, den });
  }

  function resetCrop(): void {
    customPicked = false;
    editor.updateGeometryDraftAspect({ kind: 'original' });
  }
</script>

<div class="flex flex-col gap-1 pb-1.5">
  <SectionHeader title="Crop" modified={cropModified} onReset={resetCrop}>
    {#snippet actions()}
      {#if outputSize}
        <span class="text-[11px] tabular-nums text-dark/55" data-testid="crop-output-size"
          >{outputSize.w} × {outputSize.h} px</span
        >
      {/if}
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiGrid}
        title={hint(`Crop guide: ${gridLabel}`, 'cropGrid')}
        aria-label={`Crop guide: ${gridLabel}`}
        onclick={ui.cycleCropGrid}
      />
    {/snippet}
  </SectionHeader>
  <div class="flex items-center gap-1.5">
    <Field label="Aspect Ratio" size="tiny" class="min-w-0 flex-1">
      <Select
        size="tiny"
        class="editor-compact-select editor-compact-field"
        options={aspectOptions(portrait)}
        value={selectedKey}
        onChange={onAspectChange}
      />
    </Field>
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      class="size-7 bg-neutral-800 not-disabled:hover:bg-neutral-700"
      icon={portrait ? mdiCropPortrait : mdiCropLandscape}
      title={hint(portrait ? 'Switch to landscape' : 'Switch to portrait', 'flipAspect')}
      aria-label={portrait ? 'Switch to landscape' : 'Switch to portrait'}
      disabled={!orientationAvailable}
      onclick={editor.flipGeometryAspect}
    />
  </div>
  {#if selectedKey === CUSTOM_KEY}
    <div class="panel-row h-7 items-center">
      <span class="editor-compact-label select-none">Ratio</span>
      <div class="col-span-2 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-1">
        <TextInput
          compact
          color="neutral"
          class={ratioInputClass}
          inputSize={1}
          inputmode="numeric"
          aria-label="Width"
          placeholder="Width"
          value={customValues.num}
          onchange={(e) => setCustomSide('num', e.currentTarget.value)}
        />
        <span class="text-[11px] text-dark/65 select-none" aria-hidden="true">:</span>
        <TextInput
          compact
          color="neutral"
          class={ratioInputClass}
          inputSize={1}
          inputmode="numeric"
          aria-label="Height"
          placeholder="Height"
          value={customValues.den}
          onchange={(e) => setCustomSide('den', e.currentTarget.value)}
        />
      </div>
    </div>
  {/if}
</div>
