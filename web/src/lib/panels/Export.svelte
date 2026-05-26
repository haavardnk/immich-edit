<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { listAlbums } from '$lib/api/albums';
  import { listTags } from '$lib/api/tags';
  import { toasts } from '$lib/stores/toasts.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import MultiSelect from '$lib/components/MultiSelect.svelte';
  import { mdiExport, mdiCloudUpload } from '@mdi/js';
  import type {
    ExportFormat,
    BitDepthOpt,
    PngCompressionOpt,
    TiffCompressionOpt,
    ExportOptions,
    ImmichExportOptions,
    StackPrimary
  } from '$lib/api/export';
  import type { AlbumSummary } from '$lib/types/album';
  import type { TagSummary } from '$lib/api/tags';

  type Destination = 'download' | 'immich';

  let destination: Destination = $state<Destination>('download');
  let format: ExportFormat = $state<ExportFormat>('jpeg');
  let quality: number = $state(90);
  let includeExif: boolean = $state(true);
  let bitDepth: BitDepthOpt = $state<BitDepthOpt>('8');
  let pngCompression: PngCompressionOpt = $state<PngCompressionOpt>('default');
  let tiffCompression: TiffCompressionOpt = $state<TiffCompressionOpt>('lzw');
  let lossless: boolean = $state(false);

  let albumIds: string[] = $state<string[]>([]);
  let tagIds: string[] = $state<string[]>([]);
  let favorite: boolean = $state(false);
  let stackWithOriginal: boolean = $state(false);
  let stackPrimary: StackPrimary = $state<StackPrimary>('edited');
  let filenameSuffix: string = $state('_edit');

  const FORMATS: { value: ExportFormat; label: string }[] = [
    { value: 'jpeg', label: 'JPEG' },
    { value: 'png', label: 'PNG' },
    { value: 'webp', label: 'WebP' },
    { value: 'avif', label: 'AVIF' },
    { value: 'heic', label: 'HEIC' },
    { value: 'tiff', label: 'TIFF' },
    { value: 'jxl', label: 'JPEG XL' }
  ];

  let showQuality = $derived(
    format === 'jpeg' ||
      format === 'avif' ||
      format === 'heic' ||
      (format === 'webp' && !(lossless || includeExif))
  );
  let showBitDepth = $derived(format === 'png' || format === 'tiff' || format === 'jxl');
  let showPngCompression = $derived(format === 'png');
  let showTiffCompression = $derived(format === 'tiff');
  let showLossless = $derived(format === 'webp');
  let losslessForced = $derived(format === 'webp' && includeExif);

  $effect(() => {
    if (destination !== 'immich') return;
    if (library.albums.length === 0) {
      void listAlbums()
        .then((a) => (library.albums = a.sort((x, y) => x.albumName.localeCompare(y.albumName))))
        .catch((e: unknown) => toasts.push('error', `albums: ${(e as Error).message}`));
    }
    if (library.tags.length === 0) {
      void listTags()
        .then((t) => (library.tags = t))
        .catch((e: unknown) => toasts.push('error', `tags: ${(e as Error).message}`));
    }
  });

  function baseOptions(): ExportOptions {
    return {
      format,
      quality,
      includeExif,
      bitDepth,
      pngCompression,
      tiffCompression,
      lossless: format === 'webp' ? lossless || includeExif : lossless
    };
  }

  function buildImmichOptions(): ImmichExportOptions {
    return {
      ...baseOptions(),
      albumIds,
      tagIds,
      favorite,
      stackWithOriginal,
      stackPrimary,
      filenameSuffix
    };
  }

  let isLoading = $derived(destination === 'download' ? editor.exporting : editor.exportingToImmich);
  let formatLabel = $derived(FORMATS.find((f) => f.value === format)!.label);
  let buttonLabel = $derived(
    destination === 'download' ? `Export ${formatLabel}` : `Upload ${formatLabel} to Immich`
  );
  let busyLabel = $derived(destination === 'download' ? 'Exporting…' : 'Uploading…');
</script>

<div class="flex flex-col gap-4 px-4 pt-2">
  <div class="flex gap-1 p-0.5 rounded-lg bg-immich-dark-bg/40">
    <button
      class="flex-1 py-1.5 text-xs rounded transition-colors"
      class:bg-immich-dark-primary={destination === 'download'}
      class:text-immich-dark-bg={destination === 'download'}
      class:text-immich-dark-fg={destination !== 'download'}
      onclick={() => (destination = 'download')}
    >
      Download
    </button>
    <button
      class="flex-1 py-1.5 text-xs rounded transition-colors"
      class:bg-immich-dark-primary={destination === 'immich'}
      class:text-immich-dark-bg={destination === 'immich'}
      class:text-immich-dark-fg={destination !== 'immich'}
      onclick={() => (destination = 'immich')}
    >
      To Immich
    </button>
  </div>

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
      <input type="range" class="slider-range" min={1} max={100} step={1} bind:value={quality} />
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

  <label class="flex items-center gap-2 text-xs text-immich-dark-fg/80 select-none cursor-pointer">
    <input type="checkbox" class="checkbox checkbox-xs" bind:checked={includeExif} />
    Include EXIF metadata
  </label>

  {#if destination === 'immich'}
    <div class="flex flex-col gap-3 pt-3 mt-1 border-t border-white/10">
      <div class="flex flex-col gap-1">
        <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Filename suffix</span>
        <input
          type="text"
          class="input w-full bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
          bind:value={filenameSuffix}
          placeholder="_edit"
        />
      </div>

      <label class="flex items-center gap-2 text-xs text-immich-dark-fg/80 select-none cursor-pointer">
        <input type="checkbox" class="checkbox checkbox-xs" bind:checked={favorite} />
        Mark as favorite
      </label>

      <label class="flex items-center gap-2 text-xs text-immich-dark-fg/80 select-none cursor-pointer">
        <input type="checkbox" class="checkbox checkbox-xs" bind:checked={stackWithOriginal} />
        Stack with original
      </label>

      {#if stackWithOriginal}
        <div class="flex gap-3 pl-6">
          <label class="flex items-center gap-1 text-[11px] text-immich-dark-fg/80 cursor-pointer">
            <input type="radio" class="radio radio-xs" name="stackPrimary" value="edited" bind:group={stackPrimary} />
            Edit primary
          </label>
          <label class="flex items-center gap-1 text-[11px] text-immich-dark-fg/80 cursor-pointer">
            <input type="radio" class="radio radio-xs" name="stackPrimary" value="original" bind:group={stackPrimary} />
            Original primary
          </label>
        </div>
      {/if}

      <div class="flex flex-col gap-1">
        <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Albums</span>
        <MultiSelect
          options={library.albums}
          bind:selected={albumIds}
          getId={(a: AlbumSummary) => a.id}
          getLabel={(a: AlbumSummary) => a.albumName}
          placeholder="Add album…"
        />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Tags</span>
        <MultiSelect
          options={library.tags}
          bind:selected={tagIds}
          getId={(t: TagSummary) => t.id}
          getLabel={(t: TagSummary) => t.value || t.name}
          placeholder="Add tag…"
        />
      </div>
    </div>
  {/if}

  <button
    class="flex items-center justify-center gap-2 py-2.5 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={isLoading || !editor.assetId}
    onclick={() => {
      if (destination === 'download') void editor.onExport(baseOptions());
      else void editor.onUploadToImmich(buildImmichOptions());
    }}
  >
    <Icon path={destination === 'download' ? mdiExport : mdiCloudUpload} size={16} />
    {isLoading ? busyLabel : buttonLabel}
  </button>

  {#if destination === 'immich' && editor.lastUpload}
    <div
      class="text-[11px] leading-relaxed px-3 py-2 rounded-md border"
      class:bg-emerald-950={editor.lastUpload.kind === 'success'}
      class:border-emerald-500={editor.lastUpload.kind === 'success'}
      class:text-emerald-100={editor.lastUpload.kind === 'success'}
      class:bg-amber-950={editor.lastUpload.kind === 'duplicate'}
      class:border-amber-500={editor.lastUpload.kind === 'duplicate'}
      class:text-amber-100={editor.lastUpload.kind === 'duplicate'}
      class:bg-red-950={editor.lastUpload.kind === 'error'}
      class:border-red-500={editor.lastUpload.kind === 'error'}
      class:text-red-100={editor.lastUpload.kind === 'error'}
    >
      {editor.lastUpload.message}
    </div>
  {/if}
</div>
