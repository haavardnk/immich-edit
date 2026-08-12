<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    getHealth,
    getDebugTimings,
    type HealthInfo,
    type DebugTimings
  } from '$lib/api/diagnostics';
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
      health = await getHealth();
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

  function codecLabel(ok: boolean): string {
    return ok ? 'yes' : 'missing';
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
    lines.push(
      `- HEIF codecs: hevc decode ${codecLabel(h.heif_codecs.hevc_decode)}, hevc encode ${codecLabel(h.heif_codecs.hevc_encode)}, av1 decode ${codecLabel(h.heif_codecs.av1_decode)}, av1 encode ${codecLabel(h.heif_codecs.av1_encode)}`
    );
    lines.push(
      `- Immich status: ${h.immich_status.kind}${statusCode} (${h.immich_status.message})`
    );
    lines.push(`- DB ready: ${h.db_ready} (migration ${h.db_migration_version ?? '—'})`);
    lines.push(`- Cache dir: ${cfg.cache_dir ?? '—'}`);
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
      lines.push('');
      lines.push(
        `- Preview frames: ${formatBytes(t.cache_bytes.preview_frames_used)} / ${formatBytes(t.cache_bytes.preview_frames_cap)}`
      );
      lines.push(
        `- Quality frames: ${formatBytes(t.cache_bytes.quality_frames_used)} / ${formatBytes(t.cache_bytes.quality_frames_cap)}`
      );
      lines.push(
        `- Mask rasters on disk: ${formatBytes(t.cache_bytes.rasters_disk_used)} / ${formatBytes(t.cache_bytes.rasters_disk_cap)}`
      );
      if (t.gpu_pool_bytes) {
        lines.push(`- GPU pool total: ${formatBytes(t.gpu_pool_bytes.total)}`);
      }
    } else {
      lines.push('Render timings: unavailable.');
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

<div class="flex items-center justify-end gap-2">
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

{#if loading}
  <Spinner label="Loading…" />
{:else if error}
  <p class="text-sm text-red-400">{error}</p>
{:else if health}
  <section class="space-y-2">
    <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Server</h2>
    <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs">
      <dt class="text-immich-dark-fg/50">Version</dt>
      <dd class="font-mono">{health.version}</dd>
      <dt class="text-immich-dark-fg/50">Renderer mode</dt>
      <dd class="font-mono">{health.renderer_mode}</dd>
      <dt class="text-immich-dark-fg/50">Renderer active</dt>
      <dd class="font-mono">{health.renderer_active}</dd>
      <dt class="text-immich-dark-fg/50">GPU adapter</dt>
      <dd class="font-mono">{health.gpu_adapter ?? '—'}</dd>
      <dt class="text-immich-dark-fg/50">Immich</dt>
      <dd>
        <span
          class={health.immich_status.ok
            ? 'text-emerald-400'
            : health.immich_status.kind === 'api_key_rejected'
              ? 'text-amber-300'
              : 'text-red-400'}>{health.immich_status.message}</span
        >
        <span class="font-mono text-immich-dark-fg/40"
          >{health.immich_status.kind}{health.immich_status.status_code
            ? `/${health.immich_status.status_code}`
            : ''}</span
        >
      </dd>
      <dt class="text-immich-dark-fg/50">DB ready</dt>
      <dd class={health.db_ready ? 'text-emerald-400' : 'text-red-400'}>
        {health.db_ready ? 'yes' : 'no'}
      </dd>
      <dt class="text-immich-dark-fg/50">DB migration</dt>
      <dd class="font-mono">{health.db_migration_version ?? '—'}</dd>
    </dl>
  </section>

  <section class="space-y-2">
    <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">HEIF codecs</h2>
    <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs">
      <dt class="text-immich-dark-fg/50">HEIC decode</dt>
      <dd class={health.heif_codecs.hevc_decode ? 'text-emerald-400' : 'text-red-400'}>
        {codecLabel(health.heif_codecs.hevc_decode)}
      </dd>
      <dt class="text-immich-dark-fg/50">HEIC export</dt>
      <dd class={health.heif_codecs.hevc_encode ? 'text-emerald-400' : 'text-red-400'}>
        {codecLabel(health.heif_codecs.hevc_encode)}
      </dd>
      <dt class="text-immich-dark-fg/50">AVIF decode</dt>
      <dd class={health.heif_codecs.av1_decode ? 'text-emerald-400' : 'text-red-400'}>
        {codecLabel(health.heif_codecs.av1_decode)}
      </dd>
      <dt class="text-immich-dark-fg/50">AVIF export</dt>
      <dd class={health.heif_codecs.av1_encode ? 'text-emerald-400' : 'text-red-400'}>
        {codecLabel(health.heif_codecs.av1_encode)}
      </dd>
    </dl>
    {#if !health.heif_codecs.hevc_decode || !health.heif_codecs.hevc_encode || !health.heif_codecs.av1_decode || !health.heif_codecs.av1_encode}
      <p class="text-xs text-immich-dark-fg/50">
        Missing codecs come from libheif plugin packages: libheif-plugin-libde265 and
        libheif-plugin-x265 for HEIC, libheif-plugin-dav1d and libheif-plugin-aomenc for AVIF.
      </p>
    {/if}
  </section>

  {#if timings}
    <section class="space-y-2">
      <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Render latency</h2>
      <table class="w-full text-xs">
        <thead class="text-immich-dark-fg/50 text-left">
          <tr
            ><th class="font-normal pb-1">Renderer</th><th class="font-normal pb-1">Count</th><th
              class="font-normal pb-1">p50</th
            ><th class="font-normal pb-1">p95</th><th class="font-normal pb-1">p99</th><th
              class="font-normal pb-1">max</th
            ></tr
          >
        </thead>
        <tbody class="font-mono">
          <tr
            ><td>CPU</td><td>{timings.render_latency.cpu.count}</td><td
              >{formatUs(timings.render_latency.cpu.p50_us)}</td
            ><td>{formatUs(timings.render_latency.cpu.p95_us)}</td><td
              >{formatUs(timings.render_latency.cpu.p99_us)}</td
            ><td>{formatUs(timings.render_latency.cpu.max_us)}</td></tr
          >
          <tr
            ><td>GPU</td><td>{timings.render_latency.gpu.count}</td><td
              >{formatUs(timings.render_latency.gpu.p50_us)}</td
            ><td>{formatUs(timings.render_latency.gpu.p95_us)}</td><td
              >{formatUs(timings.render_latency.gpu.p99_us)}</td
            ><td>{formatUs(timings.render_latency.gpu.max_us)}</td></tr
          >
        </tbody>
      </table>
    </section>

    <section class="space-y-2">
      <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Cache usage</h2>
      <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs font-mono">
        <dt class="text-immich-dark-fg/50 font-sans">Preview frames</dt>
        <dd>
          {formatBytes(timings.cache_bytes.preview_frames_used)} / {formatBytes(
            timings.cache_bytes.preview_frames_cap
          )}
        </dd>
        <dt class="text-immich-dark-fg/50 font-sans">Quality frames</dt>
        <dd>
          {formatBytes(timings.cache_bytes.quality_frames_used)} / {formatBytes(
            timings.cache_bytes.quality_frames_cap
          )}
        </dd>
        <dt class="text-immich-dark-fg/50 font-sans">Mask rasters on disk</dt>
        <dd>
          {formatBytes(timings.cache_bytes.rasters_disk_used)} / {formatBytes(
            timings.cache_bytes.rasters_disk_cap
          )}
        </dd>
      </dl>
    </section>

    {#if timings.gpu_pool_bytes}
      <section class="space-y-2">
        <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">GPU memory pools</h2>
        <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs font-mono">
          <dt class="text-immich-dark-fg/50 font-sans">Texture pool</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.texture_pool)}</dd>
          <dt class="text-immich-dark-fg/50 font-sans">Uniform pool</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.uniform_pool)}</dd>
          <dt class="text-immich-dark-fg/50 font-sans">Output targets</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.output_targets)}</dd>
          <dt class="text-immich-dark-fg/50 font-sans">Sharpen targets</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.sharpen_targets)}</dd>
          <dt class="text-immich-dark-fg/50 font-sans">WB cache</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.wb_cache)}</dd>
          <dt class="text-immich-dark-fg/50 font-sans">NR cache</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.nr_cache)}</dd>
          <dt class="text-immich-dark-fg/50 font-sans">Atlas cache</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.atlas_cache)}</dd>
          <dt class="text-immich-dark-fg/50 font-sans">Total</dt>
          <dd class="text-immich-dark-fg">{formatBytes(timings.gpu_pool_bytes.total)}</dd>
        </dl>
      </section>
    {/if}
  {:else}
    <p class="text-xs text-immich-dark-fg/40">Render timings are unavailable.</p>
  {/if}

  <section class="space-y-2">
    <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Configuration</h2>
    <pre
      class="text-[11px] font-mono bg-black/30 border border-white/5 rounded p-3 overflow-x-auto">{JSON.stringify(
        health.config,
        null,
        2
      )}</pre>
  </section>

  <section class="space-y-2">
    <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Resources</h2>
    <ul class="text-xs space-y-1">
      <li>
        <a
          class="text-immich-primary hover:underline"
          href="https://haavardnk.github.io/immich-edit/deploy/"
          target="_blank"
          rel="noopener">Deployment</a
        >
      </li>
      <li>
        <a
          class="text-immich-primary hover:underline"
          href="https://haavardnk.github.io/immich-edit/troubleshooting/"
          target="_blank"
          rel="noopener">Troubleshooting</a
        >
      </li>
      <li>
        <a
          class="text-immich-primary hover:underline"
          href="https://github.com/haavardnk/immich-edit/releases"
          target="_blank"
          rel="noopener">Releases</a
        >
      </li>
      <li>
        <a
          class="text-immich-primary hover:underline"
          href="https://github.com/haavardnk/immich-edit/issues/new/choose"
          target="_blank"
          rel="noopener">Report an issue</a
        >
      </li>
    </ul>
  </section>
{/if}
