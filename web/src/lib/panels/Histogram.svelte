<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import type { Histogram } from '$lib/types/preview';

  const hist = $derived(editor.meta?.histogram ?? null);
  const dims = $derived(editor.meta ? `${editor.meta.width}×${editor.meta.height}` : '');
  const renderer = $derived(editor.meta?.renderer ?? '');

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

  function show(h: Histogram | null): boolean {
    return h !== null;
  }
</script>

<div class="flex flex-col gap-1">
  <div class="bg-base-300 rounded">
    {#if !show(hist)}
      <div class="text-[11px] opacity-50 h-16 flex items-center justify-center">no data</div>
    {:else if hist}
      <svg viewBox="0 0 256 64" class="w-full h-16 block" preserveAspectRatio="none">
        <path d={path(hist.r)} fill="rgba(239,68,68,0.45)" />
        <path d={path(hist.g)} fill="rgba(34,197,94,0.45)" />
        <path d={path(hist.b)} fill="rgba(59,130,246,0.45)" />
        <path d={path(hist.l)} fill="none" stroke="rgba(229,229,229,0.75)" stroke-width="1" />
      </svg>
    {/if}
  </div>
  <div class="flex items-center justify-between text-[10px] opacity-50 font-mono">
    <span>{dims}</span>
    <span>{renderer}</span>
  </div>
</div>
