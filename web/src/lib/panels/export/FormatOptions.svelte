<script lang="ts">
  import { FORMATS, type ExportForm } from './settings';

  let { form = $bindable<ExportForm>() }: { form: ExportForm } = $props();

  let showQuality = $derived(
    form.format === 'jpeg' ||
      form.format === 'avif' ||
      form.format === 'heic' ||
      (form.format === 'webp' && !(form.lossless || form.includeExif)),
  );
  let showBitDepth = $derived(
    form.format === 'png' || form.format === 'tiff' || form.format === 'jxl',
  );
  let showPngCompression = $derived(form.format === 'png');
  let showTiffCompression = $derived(form.format === 'tiff');
  let showLossless = $derived(form.format === 'webp');
  let losslessForced = $derived(form.format === 'webp' && form.includeExif);
</script>

<div class="flex flex-col gap-1">
  <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Format</span>
  <select
    class="select bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
    bind:value={form.format}
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
      <span class="font-mono tabular-nums text-[10px] text-immich-dark-fg/50">{form.quality}</span>
    </div>
    <input type="range" class="slider-range" min={1} max={100} step={1} bind:value={form.quality} />
  </div>
{/if}

{#if showBitDepth}
  <div class="flex flex-col gap-1">
    <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Bit depth</span>
    <select
      class="select bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
      bind:value={form.bitDepth}
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
      bind:value={form.pngCompression}
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
      bind:value={form.tiffCompression}
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
      checked={losslessForced ? true : form.lossless}
      disabled={losslessForced}
      onchange={(e) => (form.lossless = (e.currentTarget as HTMLInputElement).checked)}
    />
    Lossless{losslessForced ? ' (required for EXIF)' : ''}
  </label>
{/if}

<label class="flex items-center gap-2 text-xs text-immich-dark-fg/80 select-none cursor-pointer">
  <input type="checkbox" class="checkbox checkbox-xs" bind:checked={form.includeExif} />
  Include EXIF metadata
</label>
