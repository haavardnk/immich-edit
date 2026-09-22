<script lang="ts">
  import type { DebugTimings } from '$lib/api/diagnostics';
  import { formatUs, hitRate } from '$lib/diagnostics/format';
  import { Heading, Text } from '@immich/ui';

  let { timings }: { timings: DebugTimings } = $props();

  const renderers = $derived(
    (['gpu', 'cpu'] as const)
      .map((renderer) => ({
        renderer,
        stages: timings.stages.filter((s) => s.renderer === renderer)
      }))
      .filter((group) => group.stages.length > 0)
  );
  const withGpu = $derived(timings.stages.some((s) => s.gpu !== null));
</script>

<section class="space-y-2 pt-5">
  <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Render stages</Heading>
  {#if renderers.length === 0}
    <Text size="tiny" color="muted">No renders recorded yet.</Text>
  {:else}
    {#each renderers as group (group.renderer)}
      <table class="w-full text-xs" aria-label="{group.renderer.toUpperCase()} render stages">
        <thead>
          <tr class="text-[10px] uppercase text-dark/65">
            <th class="py-1 text-left font-normal">{group.renderer.toUpperCase()} stage</th>
            <th class="py-1 text-right font-normal">p50</th>
            <th class="py-1 text-right font-normal">p95</th>
            {#if withGpu}
              <th class="py-1 text-right font-normal">GPU p50</th>
              <th class="py-1 text-right font-normal">GPU p95</th>
            {/if}
          </tr>
        </thead>
        <tbody class="divide-y divide-hairline font-mono tabular-nums">
          {#each group.stages as s (s.stage)}
            <tr>
              <td class="py-1 font-sans text-dark/65">{s.stage}</td>
              <td class="py-1 text-right">{formatUs(s.wall.p50_us)}</td>
              <td class="py-1 text-right">{formatUs(s.wall.p95_us)}</td>
              {#if withGpu}
                <td class="py-1 text-right">{s.gpu ? formatUs(s.gpu.p50_us) : '—'}</td>
                <td class="py-1 text-right">{s.gpu ? formatUs(s.gpu.p95_us) : '—'}</td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>
    {/each}
    {#if !timings.gpu_timestamps}
      <Text size="tiny" color="muted">
        GPU stage times are off. Set GPU_TIMESTAMPS=true to measure device time per stage.
      </Text>
    {/if}
  {/if}
</section>

<section class="space-y-2 pt-5">
  <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Source frames</Heading>
  <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs font-mono">
    <dt class="text-dark/65 font-sans">Original fetch</dt>
    <dd>
      p50 {formatUs(timings.frames.fetch.p50_us)} · p95 {formatUs(timings.frames.fetch.p95_us)}
    </dd>
    <dt class="text-dark/65 font-sans">Decode</dt>
    <dd>
      p50 {formatUs(timings.frames.decode.p50_us)} · p95 {formatUs(timings.frames.decode.p95_us)}
    </dd>
    <dt class="text-dark/65 font-sans">Frame cache hits</dt>
    <dd>
      {hitRate(timings.frames.cache_hits, timings.frames.cache_misses)}
      ({timings.frames.cache_hits} / {timings.frames.cache_hits + timings.frames.cache_misses})
    </dd>
    <dt class="text-dark/65 font-sans">Timed-out requests</dt>
    <dd>{timings.request_timeouts}</dd>
  </dl>
</section>
