<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';

  const exif = $derived(editor.asset?.exifInfo ?? null);

  function fmtAperture(v: number | null): string | null {
    if (v == null) return null;
    return `f/${v.toFixed(v < 10 ? 1 : 0)}`;
  }
  function fmtFocal(v: number | null): string | null {
    if (v == null) return null;
    return `${v.toFixed(0)}mm`;
  }
  function fmtShutter(v: string | null): string | null {
    if (!v) return null;
    const n = Number(v);
    if (!Number.isFinite(n) || n <= 0) return v;
    if (n >= 1) return `${n.toFixed(1)}s`;
    return `1/${Math.round(1 / n)}s`;
  }
  function fmtDim(w: number | null, h: number | null): string | null {
    if (!w || !h) return null;
    return `${w} × ${h}`;
  }
  function fmtSize(b: number | null): string | null {
    if (!b) return null;
    const mb = b / (1024 * 1024);
    if (mb >= 1) return `${mb.toFixed(1)} MB`;
    const kb = b / 1024;
    return `${kb.toFixed(0)} KB`;
  }
  function fmtDate(s: string | null): string | null {
    if (!s) return null;
    try {
      const d = new Date(s);
      if (Number.isNaN(d.getTime())) return s;
      return d.toLocaleString();
    } catch {
      return s;
    }
  }

  const rows = $derived.by(() => {
    if (!exif) return [] as Array<[string, string]>;
    const items: Array<[string, string | null]> = [
      ['Camera', [exif.make, exif.model].filter(Boolean).join(' ') || null],
      ['Lens', exif.lensModel],
      ['Aperture', fmtAperture(exif.fNumber)],
      ['Focal', fmtFocal(exif.focalLength)],
      ['Shutter', fmtShutter(exif.exposureTime)],
      ['ISO', exif.iso != null ? String(exif.iso) : null],
      ['Dimensions', fmtDim(exif.exifImageWidth, exif.exifImageHeight)],
      ['Size', fmtSize(exif.fileSizeInByte)],
      ['Taken', fmtDate(exif.dateTimeOriginal)]
    ];
    return items.filter((r): r is [string, string] => r[1] != null && r[1] !== '');
  });
</script>

<div class="flex flex-col gap-1 text-[11px]">
  {#if rows.length === 0}
    <div class="text-immich-dark-fg/30 italic">No EXIF data</div>
  {:else}
    {#each rows as [k, v] (k)}
      <div class="flex justify-between gap-2">
        <span class="text-immich-dark-fg/50">{k}</span>
        <span class="text-immich-dark-fg/90 font-mono truncate" title={v}>{v}</span>
      </div>
    {/each}
  {/if}
</div>
