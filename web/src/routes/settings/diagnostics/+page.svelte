<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getHealth,
    getDebugTimings,
    type HealthInfo,
    type DebugTimings,
    type LatencyStats
  } from '$lib/api/diagnostics';
  import Notice from '$lib/components/Notice.svelte';
  import RenderStages from '$lib/components/settings/RenderStages.svelte';
  import { codecLabel, formatBytes, formatUs } from '$lib/diagnostics/format';
  import { buildSupportBundle } from '$lib/diagnostics/supportBundle';
  import { errorMessage } from '$lib/utils/errors';
  import { Button, Heading, LoadingSpinner, Text } from '@immich/ui';

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
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  }

  async function copySupportBundle(): Promise<void> {
    if (!health) return;
    try {
      await navigator.clipboard.writeText(buildSupportBundle(health, timings, navigator.userAgent));
      copyState = 'ok';
    } catch {
      copyState = 'fail';
    }
    setTimeout(() => {
      copyState = 'idle';
    }, 2000);
  }

  onMount(() => {
    void refresh();
  });
</script>

{#snippet latencyRow(name: string, stats: LatencyStats)}
  <div class="grid items-center gap-3 py-3 sm:grid-cols-[5rem_7rem_minmax(0,1fr)]">
    <div>
      <div class="text-xs font-medium">{name}</div>
      <div class="text-[10px] text-dark/65">{stats.count} renders</div>
    </div>
    <div>
      <div class="text-[10px] uppercase text-dark/65">Typical</div>
      <div class="font-mono text-lg tabular-nums text-white/90">{formatUs(stats.p50_us)}</div>
    </div>
    <dl class="grid grid-cols-3 gap-3 text-right text-[10px]">
      <div>
        <dt class="text-dark/65">p95</dt>
        <dd class="font-mono text-xs tabular-nums text-dark">{formatUs(stats.p95_us)}</dd>
      </div>
      <div>
        <dt class="text-dark/65">p99</dt>
        <dd class="font-mono text-xs tabular-nums text-dark">{formatUs(stats.p99_us)}</dd>
      </div>
      <div>
        <dt class="text-dark/65">Peak</dt>
        <dd class="font-mono text-xs tabular-nums text-dark">{formatUs(stats.max_us)}</dd>
      </div>
    </dl>
  </div>
{/snippet}

<div class="flex items-center justify-end gap-2">
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    title="Copy a diagnostics block to paste into a bug report"
    disabled={loading || !health}
    onclick={() => void copySupportBundle()}
  >
    {copyState === 'ok' ? 'Copied' : copyState === 'fail' ? 'Copy failed' : 'Copy support bundle'}
  </Button>
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    onclick={() => void refresh()}
    disabled={loading}
  >
    Refresh
  </Button>
</div>

{#if loading}
  <div class="inline-flex items-center gap-2 text-dark/65" aria-live="polite">
    <LoadingSpinner size="small" />
    <span class="text-xs">Loading…</span>
  </div>
{:else if error}
  <Notice message={error} class="text-sm" />
{:else if health}
  <section class="space-y-2">
    <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Server</Heading>
    <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs">
      <dt class="text-dark/65">Version</dt>
      <dd class="font-mono">{health.version}</dd>
      <dt class="text-dark/65">Renderer mode</dt>
      <dd class="font-mono">{health.renderer_mode}</dd>
      <dt class="text-dark/65">Renderer active</dt>
      <dd class="font-mono">{health.renderer_active}</dd>
      <dt class="text-dark/65">GPU adapter</dt>
      <dd class="font-mono">{health.gpu_adapter ?? '—'}</dd>
      {#if health.gpu_software}
        <dt class="text-dark/65">GPU type</dt>
        <dd class="font-mono text-amber-300">software rasterizer (no hardware GPU found)</dd>
      {/if}
      <dt class="text-dark/65">Immich</dt>
      <dd>
        <span
          class={health.immich_status.ok
            ? 'text-emerald-400'
            : health.immich_status.kind === 'api_key_rejected'
              ? 'text-amber-300'
              : 'text-red-400'}>{health.immich_status.message}</span
        >
        <span class="font-mono text-dark/65"
          >{health.immich_status.kind}{health.immich_status.status_code
            ? `/${health.immich_status.status_code}`
            : ''}</span
        >
      </dd>
      <dt class="text-dark/65">DB ready</dt>
      <dd class={health.db_ready ? 'text-emerald-400' : 'text-red-400'}>
        {health.db_ready ? 'yes' : 'no'}
      </dd>
      <dt class="text-dark/65">DB migration</dt>
      <dd class="font-mono">{health.db_migration_version ?? '—'}</dd>
    </dl>
  </section>

  <section class="space-y-2 pt-5">
    <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Host</Heading>
    <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs">
      <dt class="text-dark/65">System</dt>
      <dd class="font-mono">
        {health.host.os}
        {health.host.arch}{health.host.os_version ? ` (${health.host.os_version})` : ''}
      </dd>
      <dt class="text-dark/65">CPU</dt>
      <dd class="font-mono">{health.host.cpu ?? '—'}</dd>
      <dt class="text-dark/65">Cores available</dt>
      <dd class="font-mono">{health.host.cores}</dd>
      <dt class="text-dark/65">Memory</dt>
      <dd class="font-mono">
        {health.host.memory_total_bytes ? formatBytes(health.host.memory_total_bytes) : '—'}
        {health.host.memory_limit_bytes
          ? `(${formatBytes(health.host.memory_limit_bytes)} container limit)`
          : ''}
      </dd>
    </dl>
  </section>

  <section class="space-y-2 pt-5">
    <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">HEIF codecs</Heading>
    <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs">
      <dt class="text-dark/65">HEIC decode</dt>
      <dd class={health.heif_codecs.hevc_decode ? 'text-emerald-400' : 'text-red-400'}>
        {codecLabel(health.heif_codecs.hevc_decode)}
      </dd>
      <dt class="text-dark/65">HEIC export</dt>
      <dd class={health.heif_codecs.hevc_encode ? 'text-emerald-400' : 'text-red-400'}>
        {codecLabel(health.heif_codecs.hevc_encode)}
      </dd>
      <dt class="text-dark/65">AVIF decode</dt>
      <dd class={health.heif_codecs.av1_decode ? 'text-emerald-400' : 'text-red-400'}>
        {codecLabel(health.heif_codecs.av1_decode)}
      </dd>
      <dt class="text-dark/65">AVIF export</dt>
      <dd class={health.heif_codecs.av1_encode ? 'text-emerald-400' : 'text-red-400'}>
        {codecLabel(health.heif_codecs.av1_encode)}
      </dd>
    </dl>
    {#if !health.heif_codecs.hevc_decode || !health.heif_codecs.hevc_encode || !health.heif_codecs.av1_decode || !health.heif_codecs.av1_encode}
      <Text size="tiny" color="muted">
        Missing codecs come from libheif plugin packages: libheif-plugin-libde265 and
        libheif-plugin-x265 for HEIC, libheif-plugin-dav1d and libheif-plugin-aomenc for AVIF.
      </Text>
    {/if}
  </section>

  {#if timings}
    <section class="space-y-2 pt-5">
      <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Render latency</Heading>
      <div class="divide-y divide-hairline">
        {@render latencyRow('CPU', timings.render_latency.cpu)}
        {@render latencyRow('GPU', timings.render_latency.gpu)}
      </div>
    </section>

    <RenderStages {timings} />

    <section class="space-y-2 pt-5">
      <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Cache usage</Heading>
      <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs font-mono">
        <dt class="text-dark/65 font-sans">Preview frames</dt>
        <dd>
          {formatBytes(timings.cache_bytes.preview_frames_used)} / {formatBytes(
            timings.cache_bytes.preview_frames_cap
          )}
        </dd>
        <dt class="text-dark/65 font-sans">Quality frames</dt>
        <dd>
          {formatBytes(timings.cache_bytes.quality_frames_used)} / {formatBytes(
            timings.cache_bytes.quality_frames_cap
          )}
        </dd>
        <dt class="text-dark/65 font-sans">Mask rasters on disk</dt>
        <dd>
          {formatBytes(timings.cache_bytes.rasters_disk_used)} / {formatBytes(
            timings.cache_bytes.rasters_disk_cap
          )}
        </dd>
      </dl>
    </section>

    {#if timings.gpu_pool_bytes}
      <section class="space-y-2 pt-5">
        <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">GPU memory pools</Heading>
        <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs font-mono">
          <dt class="text-dark/65 font-sans">Texture pool</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.texture_pool)}</dd>
          <dt class="text-dark/65 font-sans">Uniform pool</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.uniform_pool)}</dd>
          <dt class="text-dark/65 font-sans">Output targets</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.output_targets)}</dd>
          <dt class="text-dark/65 font-sans">Sharpen targets</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.sharpen_targets)}</dd>
          <dt class="text-dark/65 font-sans">WB cache</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.wb_cache)}</dd>
          <dt class="text-dark/65 font-sans">NR cache</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.nr_cache)}</dd>
          <dt class="text-dark/65 font-sans">Atlas cache</dt>
          <dd>{formatBytes(timings.gpu_pool_bytes.atlas_cache)}</dd>
          <dt class="text-dark/65 font-sans">Total</dt>
          <dd class="text-dark">{formatBytes(timings.gpu_pool_bytes.total)}</dd>
        </dl>
      </section>
    {/if}
  {:else}
    <Text size="tiny" color="muted">Render timings are unavailable.</Text>
  {/if}

  <section class="space-y-2 pt-5">
    <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Configuration</Heading>
    <pre
      class="overflow-x-auto rounded border border-dark/10 bg-light-100 p-3 font-mono text-[11px]">{JSON.stringify(
        health.config,
        null,
        2
      )}</pre>
  </section>

  <section class="space-y-2 pt-5">
    <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Resources</Heading>
    <ul class="text-xs space-y-1">
      <li>
        <a
          class="text-primary hover:underline"
          href="https://haavardnk.github.io/immich-edit/deploy/"
          target="_blank"
          rel="noopener">Deployment</a
        >
      </li>
      <li>
        <a
          class="text-primary hover:underline"
          href="https://haavardnk.github.io/immich-edit/troubleshooting/"
          target="_blank"
          rel="noopener">Troubleshooting</a
        >
      </li>
      <li>
        <a
          class="text-primary hover:underline"
          href="https://github.com/haavardnk/immich-edit/releases"
          target="_blank"
          rel="noopener">Releases</a
        >
      </li>
      <li>
        <a
          class="text-primary hover:underline"
          href="https://github.com/haavardnk/immich-edit/issues/new/choose"
          target="_blank"
          rel="noopener">Report an issue</a
        >
      </li>
    </ul>
  </section>
{/if}
