<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { getHealth, getDebugTimings, type HealthInfo, type DebugTimings } from '$lib/api/diagnostics';
  import Spinner from '$lib/components/Spinner.svelte';

  let health = $state<HealthInfo | null>(null);
  let timings = $state<DebugTimings | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let copyState = $state<'idle' | 'ok' | 'fail'>('idle');

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

  function buildSupportBundle(h: HealthInfo, t: DebugTimings | null): string {
    const cfg = h.config as Record<string, unknown>;
    const statusCode = h.immich_status.status_code ? ` HTTP ${h.immich_status.status_code}` : '';
    const lines: string[] = [];
    lines.push('## immich-edit support bundle');
    lines.push('');
    lines.push(`- Version: ${h.version}`);
    lines.push(`- Renderer mode: ${h.renderer_mode}`);
    lines.push(`- Renderer active: ${h.renderer_active}`);
    lines.push(`- GPU adapter: ${h.gpu_adapter ?? 'none'}`);
    lines.push(`- Immich status: ${h.immich_status.kind}${statusCode} (${h.immich_status.message})`);
    lines.push(`- DB ready: ${h.db_ready} (migration ${h.db_migration_version ?? '—'})`);
    lines.push(`- Cache dir: ${cfg.cache_dir ?? '—'}`);
    lines.push(`- Debug endpoints: ${cfg.debug_endpoints ?? false}`);
    lines.push(`- User agent: ${navigator.userAgent}`);
    lines.push('');
    if (t) {
      lines.push('### Render latency');
      lines.push('');
      lines.push('| Renderer | Count | p50 | p95 | p99 | max |');
      lines.push('|---|---|---|---|---|---|');
      const row = (name: string, s: typeof t.render_latency.cpu) =>
        `| ${name} | ${s.count} | ${formatUs(s.p50_us)} | ${formatUs(s.p95_us)} | ${formatUs(s.p99_us)} | ${formatUs(s.max_us)} |`;
      lines.push(row('cpu', t.render_latency.cpu));
      lines.push(row('gpu', t.render_latency.gpu));
      if (t.gpu_pool_bytes) {
        lines.push('');
        lines.push(`- GPU pool total: ${formatBytes(t.gpu_pool_bytes.total)}`);
      }
    } else {
      lines.push('Render timings: unavailable (debug endpoints disabled).');
    }
    lines.push('');
    lines.push('### Redacted config');
    lines.push('');
    lines.push('```json');
    lines.push(JSON.stringify(h.config, null, 2));
    lines.push('```');
    return lines.join('\n');
  }

  async function copySupportBundle(): Promise<void> {
    if (!health) return;
    try {
      await navigator.clipboard.writeText(buildSupportBundle(health, timings));
      copyState = 'ok';
    } catch {
      copyState = 'fail';
    }
    setTimeout(() => {
      copyState = 'idle';
    }, 2000);
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
      <div class="flex items-center gap-2">
        <button
          class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs disabled:opacity-50"
          onclick={() => void copySupportBundle()}
          disabled={loading || !health}
          title="Copy a diagnostics block to paste into a bug report"
        >
          {copyState === 'ok' ? 'Copied' : copyState === 'fail' ? 'Copy failed' : 'Copy support bundle'}
        </button>
        <button
          class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs"
          onclick={() => void refresh()}
          disabled={loading}
        >
          Refresh
        </button>
      </div>
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
          <dt class="text-immich-dark-fg/50">Immich</dt><dd><span class={health.immich_status.ok ? 'text-emerald-400' : health.immich_status.kind === 'api_key_rejected' ? 'text-amber-300' : 'text-red-400'}>{health.immich_status.message}</span> <span class="font-mono text-immich-dark-fg/40">{health.immich_status.kind}{health.immich_status.status_code ? `/${health.immich_status.status_code}` : ''}</span></dd>
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

      <section class="space-y-2">
        <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Resources</h2>
        <ul class="text-xs space-y-1">
          <li><a class="text-immich-primary hover:underline" href="https://github.com/haavardnk/immich-edit/blob/main/docs/deploy.md" target="_blank" rel="noopener">Deployment & troubleshooting</a></li>
          <li><a class="text-immich-primary hover:underline" href="https://github.com/haavardnk/immich-edit/blob/main/CHANGELOG.md" target="_blank" rel="noopener">Changelog</a></li>
          <li><a class="text-immich-primary hover:underline" href="https://github.com/haavardnk/immich-edit/issues/new/choose" target="_blank" rel="noopener">Report an issue</a></li>
        </ul>
      </section>
    {/if}
  </div>
</div>
