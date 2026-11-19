<script lang="ts">
  import { watermarkPngUrl } from '#lib/api/watermarks.ts';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
  import { watermarkBox } from '#lib/panels/export/watermark.ts';
  import { watermarkLibrary } from '#lib/panels/export/watermarkLibrary.svelte.ts';
  import { watermarkPreview } from '#lib/panels/export/watermarkPreview.svelte.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';

  let { img }: { img: PreviewSurface | null } = $props();

  const rect = imageRect(() => img);
  const form = $derived(exportSettings.form);
  const mark = $derived(
    watermarkPreview.showing && form.watermarkId
      ? (watermarkLibrary.items.find((item) => item.id === form.watermarkId) ?? null)
      : null
  );
  const box = $derived(
    mark && rect.w > 0 && rect.h > 0
      ? watermarkBox({ w: rect.w, h: rect.h }, mark, {
          size: form.watermarkSize,
          inset: form.watermarkInset,
          anchor: form.watermarkAnchor
        })
      : null
  );
</script>

{#if mark && box}
  <img
    src={watermarkPngUrl(mark.id)}
    alt=""
    data-testid="watermark-preview"
    class="pointer-events-none absolute max-w-none select-none"
    style:left="{rect.x + box.left}px"
    style:top="{rect.y + box.top}px"
    style:width="{box.width}px"
    style:height="{box.height}px"
    style:opacity={form.watermarkOpacity / 100}
  />
{/if}
