<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { getHealth, getDebugTimings, type HealthInfo, type DebugTimings } from '$lib/api/diagnostics';
  import Spinner from '$lib/components/Spinner.svelte';

  let health = $state<HealthInfo | null>(null);
  let timings = $state<DebugTimings | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function refresh(): Promise<void> {
    loading = true;
    error = null;
    try {
      const h = await getHealth();
      health = h;
      try {
        timings = await getDebugTimings();
      } catch {
        timings = null;
      }
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  function formatUs(us: number): string {
    if (us === 0) return '—';
    if (us < 1000) return `${us}µs`;
    return `${(us / 1000).toFixed(1)}ms`;
  }

  function formatBytes(b: number): string {
    if (b < 1024) return `${b}B`;
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)}KiB`;
    if (b < 1024 * 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)}MiB`;
    return `${(b / 1024 / 1024 / 1024).toFixed(2)}GiB`;
  }

  onMount(() => {
    editor.unload();
    void refresh();
  });
</script>

<div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden p-6">
  <div class="max-w-3xl mx-auto space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-lg font-medium">Settings &amp; diagnostics</h1>
      <button
        class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs"
        onclick={() => void refresh()}
        disabled={loading}
      >
        Refresh
      </button>
    </div>

    {#if loading}
      <Spinner label="Loading…" />
    {:else if error}
      <p class="text-sm text-red-400">{error}</p>
    {:else if health}
      <section class="space-y-2">
        <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Server</h2>
        <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs">
          <dt class="text-immich-dark-fg/50">Version</dt><dd class="font-mono">{health.version}</dd>
          <dt class="text-immich-dark-fg/50">Renderer mode</dt><dd class="font-mono">{health.renderer_mode}</dd>
          <dt class="text-immich-dark-fg/50">Renderer active</dt><dd class="font-mono">{health.renderer_active}</dd>
          <dt class="text-immich-dark-fg/50">GPU adapter</dt><dd class="font-mono">{health.gpu_adapter ?? '—'}</dd>
          <dt class="text-immich-dark-fg/50">Immich reachable</dt><dd class={health.immich_reachable ? 'text-emerald-400' : 'text-red-400'}>{health.immich_reachable ? 'yes' : 'no'}</dd>
          <dt class="text-immich-dark-fg/50">DB ready</dt><dd class={health.db_ready ? 'text-emerald-400' : 'text-red-400'}>{health.db_ready ? 'yes' : 'no'}</dd>
          <dt class="text-immich-dark-fg/50">DB migration</dt><dd class="font-mono">{health.db_migration_version ?? '—'}</dd>
        </dl>
      </section>

      <section class="space-y-2">
        <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Configuration</h2>
        <pre class="text-[11px] font-mono bg-black/30 border border-white/5 rounded p-3 overflow-x-auto">{JSON.stringify(health.config, null, 2)}</pre>
      </section>

      {#if timings}
        <section class="space-y-2">
          <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Render latency</h2>
          <table class="w-full text-xs">
            <thead class="text-immich-dark-fg/50 text-left">
              <tr><th class="font-normal pb-1">Renderer</th><th class="font-normal pb-1">Count</th><th class="font-normal pb-1">p50</th><th class="font-normal pb-1">p95</th><th class="font-normal pb-1">p99</th><th class="font-normal pb-1">max</th></tr>
            </thead>
            <tbody class="font-mono">
              <tr><td>CPU</td><td>{timings.render_latency.cpu.count}</td><td>{formatUs(timings.render_latency.cpu.p50_us)}</td><td>{formatUs(timings.render_latency.cpu.p95_us)}</td><td>{formatUs(timings.render_latency.cpu.p99_us)}</td><td>{formatUs(timings.render_latency.cpu.max_us)}</td></tr>
              <tr><td>GPU</td><td>{timings.render_latency.gpu.count}</td><td>{formatUs(timings.render_latency.gpu.p50_us)}</td><td>{formatUs(timings.render_latency.gpu.p95_us)}</td><td>{formatUs(timings.render_latency.gpu.p99_us)}</td><td>{formatUs(timings.render_latency.gpu.max_us)}</td></tr>
            </tbody>
          </table>
        </section>

        {#if timings.gpu_pool_bytes}
          <section class="space-y-2">
            <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">GPU memory pools</h2>
            <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs font-mono">
              <dt class="text-immich-dark-fg/50 font-sans">Texture pool</dt><dd>{formatBytes(timings.gpu_pool_bytes.texture_pool)}</dd>
              <dt class="text-immich-dark-fg/50 font-sans">Uniform pool</dt><dd>{formatBytes(timings.gpu_pool_bytes.uniform_pool)}</dd>
              <dt class="text-immich-dark-fg/50 font-sans">Output targets</dt><dd>{formatBytes(timings.gpu_pool_bytes.output_targets)}</dd>
              <dt class="text-immich-dark-fg/50 font-sans">Sharpen targets</dt><dd>{formatBytes(timings.gpu_pool_bytes.sharpen_targets)}</dd>
              <dt class="text-immich-dark-fg/50 font-sans">WB cache</dt><dd>{formatBytes(timings.gpu_pool_bytes.wb_cache)}</dd>
              <dt class="text-immich-dark-fg/50 font-sans">NR cache</dt><dd>{formatBytes(timings.gpu_pool_bytes.nr_cache)}</dd>
              <dt class="text-immich-dark-fg/50 font-sans">Atlas cache</dt><dd>{formatBytes(timings.gpu_pool_bytes.atlas_cache)}</dd>
              <dt class="text-immich-dark-fg/50 font-sans">Total</dt><dd class="text-immich-dark-fg">{formatBytes(timings.gpu_pool_bytes.total)}</dd>
            </dl>
          </section>
        {/if}
      {:else}
        <p class="text-xs text-immich-dark-fg/40">Debug timings disabled (set <code class="font-mono">IMMICH_EDIT_DEBUG_ENDPOINTS=true</code> to enable).</p>
      {/if}
    {/if}
  </div>
</div>
