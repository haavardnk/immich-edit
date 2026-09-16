<script lang="ts">
  import type { Histogram } from '$lib/types/preview';

  let { hist, linear, gain }: { hist: Histogram; linear: Histogram | null; gain: number } =
    $props();

  const W = 256;
  const H = 100;
  const levels = [0, 25, 50, 75, 100];

  function path(values: number[], gain: number): string {
    if (values.length === 0) return '';
    const max = Math.max(...values, 1);
    const n = values.length;
    let d = `M 0 ${H}`;
    for (let i = 0; i < n; i++) {
      const x = (i / (n - 1)) * W;
      const y = H - Math.min(1, ((values[i] ?? 0) / max) * gain) * H;
      d += ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
    }
    d += ` L ${W} ${H} Z`;
    return d;
  }

  function clippingPct(h: Histogram, bin: number): number {
    const total = h.l.reduce((a, b) => a + b, 0);
    if (total === 0) return 0;
    return (((h.r[bin] ?? 0) + (h.g[bin] ?? 0) + (h.b[bin] ?? 0)) / (total * 3)) * 100;
  }

  const shadowClip = $derived(linear ? clippingPct(linear, 0) : 0);
  const highlightClip = $derived(linear ? clippingPct(linear, 255) : 0);
</script>

<div class="relative h-32 w-full bg-neutral-950" role="img" aria-label="Histogram">
  <svg
    viewBox="0 0 {W} {H}"
    preserveAspectRatio="none"
    aria-hidden="true"
    class="absolute inset-0 h-full w-full"
  >
    <path
      d={path(hist.r, gain)}
      fill="color-mix(in srgb, var(--color-channel-red) 45%, transparent)"
    />
    <path
      d={path(hist.g, gain)}
      fill="color-mix(in srgb, var(--color-channel-green) 45%, transparent)"
    />
    <path
      d={path(hist.b, gain)}
      fill="color-mix(in srgb, var(--color-channel-blue) 45%, transparent)"
    />
    <path
      d={path(hist.l, gain)}
      fill="none"
      stroke="color-mix(in srgb, var(--color-channel-luma) 60%, transparent)"
      stroke-width="1"
      vector-effect="non-scaling-stroke"
    />
  </svg>
  <svg
    viewBox="0 0 100 100"
    preserveAspectRatio="none"
    aria-hidden="true"
    class="pointer-events-none absolute inset-0 h-full w-full"
  >
    {#each levels as level (level)}
      <line
        x1={level}
        x2={level}
        y1="0"
        y2="100"
        stroke="rgba(255,255,255,0.14)"
        stroke-width="0.4"
      />
    {/each}
  </svg>
  <div
    class="pointer-events-none absolute inset-x-0 bottom-0 flex justify-between px-0.5 text-[8px] font-mono text-dark/45"
  >
    {#each levels as level (level)}
      <span>{level}</span>
    {/each}
  </div>
  {#if shadowClip > 0.1}
    <div class="absolute left-1 top-0.5 text-[9px] font-mono text-blue-400" title="Shadow clipping">
      ▼
    </div>
  {/if}
  {#if highlightClip > 0.1}
    <div
      class="absolute right-1 top-0.5 text-[9px] font-mono text-red-400"
      title="Highlight clipping"
    >
      ▲
    </div>
  {/if}
</div>
