<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import type { Histogram } from '$lib/types/preview';

  const hist = $derived(editor.meta?.histogram ?? null);
  const linearHist = $derived(editor.meta?.linear_histogram ?? null);

  function path(values: number[]): string {
    if (values.length === 0) return '';
    const max = Math.max(...values, 1);
    const n = values.length;
    const w = 256;
    const h = 64;
    let d = `M 0 ${h}`;
    for (let i = 0; i < n; i++) {
      const x = (i / (n - 1)) * w;
      const y = h - (values[i] / max) * h;
      d += ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
    }
    d += ` L ${w} ${h} Z`;
    return d;
  }

  function clippingPct(h: Histogram, bin: number): number {
    const total = h.l.reduce((a, b) => a + b, 0);
    if (total === 0) return 0;
    return ((h.r[bin] + h.g[bin] + h.b[bin]) / (total * 3)) * 100;
  }

  const shadowClip = $derived(linearHist ? clippingPct(linearHist, 0) : 0);
  const highlightClip = $derived(linearHist ? clippingPct(linearHist, 255) : 0);
</script>

<div class="relative overflow-hidden bg-neutral-950">
  {#if editor.meta === null}
    <div role="status" class="flex h-14 items-center justify-center text-[10px] text-dark/65">
      Loading histogram…
    </div>
  {:else if hist === null}
    <div class="flex h-14 items-center justify-center text-[10px] text-dark/65">
      No histogram data
    </div>
  {:else}
    <svg viewBox="0 0 256 64" class="block h-14 w-full" preserveAspectRatio="none">
      <path d={path(hist.r)} fill="color-mix(in srgb, var(--color-channel-red) 45%, transparent)" />
      <path
        d={path(hist.g)}
        fill="color-mix(in srgb, var(--color-channel-green) 45%, transparent)"
      />
      <path
        d={path(hist.b)}
        fill="color-mix(in srgb, var(--color-channel-blue) 45%, transparent)"
      />
      <path
        d={path(hist.l)}
        fill="none"
        stroke="color-mix(in srgb, var(--color-channel-luma) 60%, transparent)"
        stroke-width="1"
      />
    </svg>
    {#if shadowClip > 0.1}
      <div
        class="absolute bottom-0.5 left-1 text-[9px] font-mono text-blue-400"
        title="Shadow clipping"
      >
        ▼
      </div>
    {/if}
    {#if highlightClip > 0.1}
      <div
        class="absolute bottom-0.5 right-1 text-[9px] font-mono text-red-400"
        title="Highlight clipping"
      >
        ▲
      </div>
    {/if}
  {/if}
</div>
