<script lang="ts">
  import type { Histogram } from '$lib/types/preview';

  let { hist }: { hist: Histogram | null } = $props();

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
</script>

<div class="bg-base-300 rounded p-2">
  {#if !hist}
    <div class="text-xs opacity-50 h-16 flex items-center justify-center">no data</div>
  {:else}
    <svg viewBox="0 0 256 64" class="w-full h-16" preserveAspectRatio="none">
      <path d={path(hist.r)} fill="rgba(239,68,68,0.5)" />
      <path d={path(hist.g)} fill="rgba(34,197,94,0.5)" />
      <path d={path(hist.b)} fill="rgba(59,130,246,0.5)" />
      <path d={path(hist.l)} fill="none" stroke="rgba(229,229,229,0.8)" stroke-width="1" />
    </svg>
  {/if}
</div>
