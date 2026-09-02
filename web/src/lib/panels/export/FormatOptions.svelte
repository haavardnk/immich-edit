<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import type {
    BitDepthOpt,
    ColorSpaceOpt,
    PngCompressionOpt,
    TiffCompressionOpt
  } from '$lib/api/export';
  import { Field, Select } from '@immich/ui';
  import RangeSlider from '$lib/components/editor/controls/RangeSlider.svelte';
  import { FORMATS, type ExportForm } from './settings';

  let { form = $bindable<ExportForm>() }: { form: ExportForm } = $props();

  const colorSpaces: { value: ColorSpaceOpt; label: string }[] = [
    { value: 'srgb', label: 'sRGB' },
    { value: 'displayp3', label: 'Display P3' }
  ];
  const bitDepths: { value: BitDepthOpt; label: string }[] = [
    { value: '8', label: '8-bit' },
    { value: '16', label: '16-bit' }
  ];
  const pngCompressions: { value: PngCompressionOpt; label: string }[] = [
    { value: 'fast', label: 'Fast' },
    { value: 'default', label: 'Default' },
    { value: 'best', label: 'Best' }
  ];
  const tiffCompressions: { value: TiffCompressionOpt; label: string }[] = [
    { value: 'none', label: 'None' },
    { value: 'lzw', label: 'LZW' },
    { value: 'deflate', label: 'Deflate' }
  ];

  let showQuality = $derived(
    form.format === 'jpeg' ||
      form.format === 'avif' ||
      form.format === 'heic' ||
      (form.format === 'webp' && !(form.lossless || form.includeExif))
  );
  let showBitDepth = $derived(
    form.format === 'png' || form.format === 'tiff' || form.format === 'jxl'
  );
  let showPngCompression = $derived(form.format === 'png');
  let showTiffCompression = $derived(form.format === 'tiff');
  let showLossless = $derived(form.format === 'webp');
  let losslessForced = $derived(form.format === 'webp' && form.includeExif);
</script>

<Field label="Format" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={FORMATS}
    value={form.format}
    onChange={(v) => (form.format = v)}
  />
</Field>

<Field label="Color space" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={colorSpaces}
    value={form.colorSpace}
    onChange={(v) => (form.colorSpace = v)}
  />
</Field>

{#if showQuality}
  <div class="panel-row h-7 items-center">
    <span class="editor-compact-label select-none">Quality</span>
    <RangeSlider
      label="Quality"
      min={1}
      max={100}
      step={1}
      value={form.quality}
      oninput={(event) => {
        form.quality = (event.currentTarget as HTMLInputElement).valueAsNumber;
      }}
    />
    <span class="px-1 text-right font-mono text-[10px] tabular-nums text-dark/65"
      >{form.quality}</span
    >
  </div>
{/if}

{#if showBitDepth}
  <Field label="Bit depth" size="tiny">
    <Select
      size="tiny"
      class="editor-compact-select editor-compact-field"
      options={bitDepths}
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
      options={pngCompressions}
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
      options={tiffCompressions}
      value={form.tiffCompression}
      onChange={(v) => (form.tiffCompression = v)}
    />
  </Field>
{/if}

{#if showLossless}
  <CheckboxRow
    label={`Lossless${losslessForced ? ' (required for EXIF)' : ''}`}
    checked={losslessForced ? true : form.lossless}
    disabled={losslessForced}
    onChange={(v) => (form.lossless = v)}
  />
{/if}

<CheckboxRow
  label="Include EXIF metadata"
  checked={form.includeExif}
  onChange={(v) => (form.includeExif = v)}
/>
