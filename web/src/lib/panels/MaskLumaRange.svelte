<script lang="ts">
  import DualRange from '#lib/components/editor/controls/DualRange.svelte';
  import { LUMA_MAX, LUMA_MIN } from '#lib/editor/params/maskComponents.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  let {
    min,
    max,
    softness,
    onLive,
    onCommit
  }: {
    min: number;
    max: number;
    softness: number;
    onLive: (value: [number, number]) => void;
    onCommit: () => void;
  } = $props();

  const W = 256;
  const H = 40;

  const bins = $derived<readonly number[]>(editor.meta?.histogram?.l ?? []);
  const curve = $derived.by(() => {
    if (bins.length < 2) return '';
    const peak = Math.max(...bins, 1);
    const step = W / (bins.length - 1);
    const points = bins.map(
      (count, i) => `L ${(i * step).toFixed(1)} ${(H - (count / peak) * H).toFixed(1)}`
    );
    return `M 0 ${H} ${points.join(' ')} L ${W} ${H} Z`;
  });
  const band = $derived(
    [
      [(min - softness) * W, H],
      [min * W, 2],
      [max * W, 2],
      [(max + softness) * W, H]
    ]
      .map(([x, y]) => `${Math.max(0, Math.min(W, x ?? 0)).toFixed(1)},${y}`)
      .join(' ')
  );
</script>

<div class="flex flex-col gap-1 py-1" data-testid="luma-range">
  <svg
    viewBox="0 0 {W} {H}"
    preserveAspectRatio="none"
    aria-hidden="true"
    class="h-10 w-full rounded-sm bg-plot-surface"
  >
    {#if curve}
      <path d={curve} class="fill-dark/20" />
    {/if}
    <polygon
      points={band}
      class="fill-primary/25 stroke-primary"
      stroke-width="1"
      vector-effect="non-scaling-stroke"
    />
  </svg>
  <DualRange
    value={[min, max]}
    thumbLabels={['Darkest', 'Brightest']}
    defaultValue={[LUMA_MIN.defaultValue, LUMA_MAX.defaultValue]}
    trackClass="h-1.5! [background:var(--gradient-curve-luma)]! [&_.dual-range-fill]:hidden"
    valueText={(v) => LUMA_MIN.format(v)}
    {onLive}
    {onCommit}
  />
  <div class="flex items-center justify-between text-[10px] text-dark/65">
    <span>Min <span class="font-mono text-dark">{LUMA_MIN.format(min)}</span></span>
    <span>Luminance range</span>
    <span>Max <span class="font-mono text-dark">{LUMA_MAX.format(max)}</span></span>
  </div>
</div>
