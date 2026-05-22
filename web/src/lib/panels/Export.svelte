<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiExport } from '@mdi/js';
  import type {
    ExportFormat,
    BitDepthOpt,
    PngCompressionOpt,
    TiffCompressionOpt,
    ExportOptions
  } from '$lib/api/export';

  let format: ExportFormat = $state<ExportFormat>('jpeg');
  let quality: number = $state(90);
  let includeExif: boolean = $state(true);
  let bitDepth: BitDepthOpt = $state<BitDepthOpt>('8');
  let pngCompression: PngCompressionOpt = $state<PngCompressionOpt>('default');
  let tiffCompression: TiffCompressionOpt = $state<TiffCompressionOpt>('lzw');
  let lossless: boolean = $state(false);
  let speed: number = $state(6);

  const FORMATS: { value: ExportFormat; label: string }[] = [
    { value: 'jpeg', label: 'JPEG' },
    { value: 'png', label: 'PNG' },
    { value: 'webp', label: 'WebP' },
    { value: 'avif', label: 'AVIF' },
    { value: 'tiff', label: 'TIFF' },
    { value: 'jxl', label: 'JPEG XL' }
  ];

  let showQuality = $derived(
    format === 'jpeg' || format === 'avif' || (format === 'webp' && !(lossless || includeExif))
  );
  let showBitDepth = $derived(format === 'png' || format === 'tiff' || format === 'jxl');
  let showPngCompression = $derived(format === 'png');
  let showTiffCompression = $derived(format === 'tiff');
  let showLossless = $derived(format === 'webp');
  let showSpeed = $derived(format === 'avif');
  let losslessForced = $derived(format === 'webp' && includeExif);

  function buildOptions(): ExportOptions {
    return {
      format,
      quality,
      includeExif,
      bitDepth,
      pngCompression,
      tiffCompression,
      lossless: format === 'webp' ? lossless || includeExif : lossless,
      speed
    };
  }

  let buttonLabel = $derived('Export ' + FORMATS.find((f) => f.value === format)!.label);
</script>

<div class="flex flex-col gap-4 px-4 pt-2">
  <div class="flex flex-col gap-1">
    <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Format</span>
    <select
      class="select bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
      bind:value={format}
    >
      {#each FORMATS as f (f.value)}
        <option value={f.value}>{f.label}</option>
      {/each}
    </select>
  </div>

  {#if showQuality}
    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between text-[11px] leading-none">
        <span class="text-immich-dark-fg/60 select-none">Quality</span>
        <span class="font-mono tabular-nums text-[10px] text-immich-dark-fg/50">{quality}</span>
      </div>
      <input
        type="range"
        class="slider-range"
        min={1}
        max={100}
        step={1}
        bind:value={quality}
      />
    </div>
  {/if}

  {#if showBitDepth}
    <div class="flex flex-col gap-1">
      <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Bit depth</span>
      <select
        class="select bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
        bind:value={bitDepth}
      >
        <option value="8">8-bit</option>
        <option value="16">16-bit</option>
      </select>
    </div>
  {/if}

  {#if showPngCompression}
    <div class="flex flex-col gap-1">
      <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Compression</span>
      <select
        class="select bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
        bind:value={pngCompression}
      >
        <option value="fast">Fast</option>
        <option value="default">Default</option>
        <option value="best">Best</option>
      </select>
    </div>
  {/if}

  {#if showTiffCompression}
    <div class="flex flex-col gap-1">
      <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Compression</span>
      <select
        class="select bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
        bind:value={tiffCompression}
      >
        <option value="none">None</option>
        <option value="lzw">LZW</option>
        <option value="deflate">Deflate</option>
      </select>
    </div>
  {/if}

  {#if showLossless}
    <label class="flex items-center gap-2 text-xs text-immich-dark-fg/80 select-none cursor-pointer">
      <input
        type="checkbox"
        class="checkbox checkbox-xs"
        checked={losslessForced ? true : lossless}
        disabled={losslessForced}
        onchange={(e) => (lossless = (e.currentTarget as HTMLInputElement).checked)}
      />
      Lossless{losslessForced ? ' (required for EXIF)' : ''}
    </label>
  {/if}

  {#if showSpeed}
    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between text-[11px] leading-none">
        <span class="text-immich-dark-fg/60 select-none">Speed</span>
        <span class="font-mono tabular-nums text-[10px] text-immich-dark-fg/50">{speed}</span>
      </div>
      <input
        type="range"
        class="slider-range"
        min={1}
        max={10}
        step={1}
        bind:value={speed}
      />
    </div>
  {/if}

  <label class="flex items-center gap-2 text-xs text-immich-dark-fg/80 select-none cursor-pointer">
    <input type="checkbox" class="checkbox checkbox-xs" bind:checked={includeExif} />
    Include EXIF metadata
  </label>

  <button
    class="flex items-center justify-center gap-2 py-2.5 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={editor.exporting}
    onclick={() => void editor.onExport(buildOptions())}
  >
    <Icon path={mdiExport} size={16} />
    {editor.exporting ? 'Exporting…' : buttonLabel}
  </button>
</div>
