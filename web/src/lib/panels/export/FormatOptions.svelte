<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import { Field, Select } from '@immich/ui';
  import ExportSlider from '#lib/panels/export/ExportSlider.svelte';
  import {
    BIT_DEPTHS,
    bitDepthShown,
    changeFormat,
    COLOR_SPACES,
    FORMATS,
    METADATA_OPTIONS,
    PNG_COMPRESSIONS,
    qualityShown,
    TIFF_COMPRESSIONS,
    type ExportFormValues
  } from './settings';

  let { form = $bindable<ExportFormValues>() }: { form: ExportFormValues } = $props();

  let showQuality = $derived(qualityShown(form));
  let showBitDepth = $derived(bitDepthShown(form.format));
  let showPngCompression = $derived(form.format === 'png');
  let showTiffCompression = $derived(form.format === 'tiff');
  let showLossless = $derived(form.format === 'webp');
</script>

<Field label="Format" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={FORMATS}
    value={form.format}
    onChange={(v) => changeFormat(form, v)}
  />
</Field>

{#if showQuality}
  <ExportSlider
    label="Quality"
    min={1}
    max={100}
    value={form.quality}
    onvalue={(v) => (form.quality = v)}
  />
{/if}

{#if showBitDepth}
  <Field label="Bit depth" size="tiny">
    <Select
      size="tiny"
      class="editor-compact-select editor-compact-field"
      options={BIT_DEPTHS}
      value={form.bitDepth}
      onChange={(v) => (form.bitDepth = v)}
    />
  </Field>
{/if}

{#if showPngCompression}
  <Field label="Compression" size="tiny">
    <Select
      size="tiny"
      class="editor-compact-select editor-compact-field"
      options={PNG_COMPRESSIONS}
      value={form.pngCompression}
      onChange={(v) => (form.pngCompression = v)}
    />
  </Field>
{/if}

{#if showTiffCompression}
  <Field label="Compression" size="tiny">
    <Select
      size="tiny"
      class="editor-compact-select editor-compact-field"
      options={TIFF_COMPRESSIONS}
      value={form.tiffCompression}
      onChange={(v) => (form.tiffCompression = v)}
    />
  </Field>
{/if}

{#if showLossless}
  <CheckboxRow label="Lossless" checked={form.lossless} onChange={(v) => (form.lossless = v)} />
{/if}

<Field label="Color space" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={COLOR_SPACES}
    value={form.colorSpace}
    onChange={(v) => (form.colorSpace = v)}
  />
</Field>

<Field label="Metadata" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={METADATA_OPTIONS}
    value={form.metadata}
    onChange={(v) => (form.metadata = v)}
  />
</Field>
