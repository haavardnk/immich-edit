<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { session } from '$lib/stores/session.svelte';
  import { logout } from '$lib/api/auth';
  import { listSessions, revokeSession, revokeAllSessions, type SessionInfo } from '$lib/api/account';
  import {
    listUsers,
    setUserAccess,
    purgeUserData,
    getInstance,
    rebindInstance,
    type AdminUser,
    type InstanceInfo
  } from '$lib/api/admin';
  import { getHealth, getDebugTimings, type HealthInfo, type DebugTimings } from '$lib/api/diagnostics';
  import {
    listMaskModels,
    installMaskModel,
    removeMaskModel,
    type MaskModel,
    type MaskModelsResponse
  } from '$lib/api/masks';
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
    void loadSessions();
    if (session.isAdmin) {
      void loadAdmin();
    }
  });

  let sessions = $state<SessionInfo[]>([]);
  let sessionsLoading = $state(false);
  let signingOut = $state(false);

  async function loadSessions(): Promise<void> {
    sessionsLoading = true;
    try {
      sessions = await listSessions();
    } catch {
      sessions = [];
    } finally {
      sessionsLoading = false;
    }
  }

  async function revokeOne(id: string): Promise<void> {
    await revokeSession(id);
    await loadSessions();
  }

  async function revokeOthers(): Promise<void> {
    await revokeAllSessions();
    await loadSessions();
  }

  function formatWhen(iso: string | null): string {
    if (!iso) return '—';
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return d.toLocaleString();
  }

  async function signOut(): Promise<void> {
    if (signingOut) return;
    signingOut = true;
    try {
      await logout();
    } catch {
      /* ignore */
    }
    session.clear();
    window.location.replace('/login');
  }

  let users = $state<AdminUser[]>([]);
  let usersLoading = $state(false);
  let adminError = $state<string | null>(null);
  let busyUser = $state<string | null>(null);
  let purgeTarget = $state<AdminUser | null>(null);
  let purgeConfirm = $state('');
  let instance = $state<InstanceInfo | null>(null);

  let showRebind = $state(false);
  let rebindUrl = $state('');
  let rebindMethod = $state<'password' | 'apikey'>('password');
  let rebindEmail = $state('');
  let rebindPassword = $state('');
  let rebindApiKey = $state('');
  let rebindConfirm = $state('');
  let rebindBusy = $state(false);
  let rebindError = $state<string | null>(null);

  async function loadAdmin(): Promise<void> {
    usersLoading = true;
    adminError = null;
    try {
      users = await listUsers();
      instance = await getInstance();
    } catch (e) {
      adminError = (e as Error).message;
    } finally {
      usersLoading = false;
    }
    await loadModels();
  }

  let models = $state<MaskModelsResponse | null>(null);
  let modelsError = $state<string | null>(null);
  let busyModel = $state<string | null>(null);

  async function loadModels(): Promise<void> {
    try {
      models = await listMaskModels();
    } catch (e) {
      modelsError = (e as Error).message;
    }
  }

  async function toggleModel(m: MaskModel): Promise<void> {
    if (busyModel) return;
    busyModel = m.id;
    modelsError = null;
    try {
      if (m.installed) {
        await removeMaskModel(m.id);
      } else {
        await installMaskModel(m.id);
      }
      await loadModels();
    } catch (e) {
      modelsError = (e as Error).message;
    } finally {
      busyModel = null;
    }
  }

  function formatMb(bytes: number): string {
    return `${Math.round(bytes / 1_000_000)} MB`;
  }

  async function toggleAccess(u: AdminUser): Promise<void> {
    if (busyUser) return;
    busyUser = u.id;
    adminError = null;
    try {
      await setUserAccess(u.id, !u.access_enabled);
      await loadAdmin();
    } catch (e) {
      adminError = (e as Error).message;
    } finally {
      busyUser = null;
    }
  }

  async function confirmPurge(): Promise<void> {
    if (!purgeTarget || busyUser) return;
    busyUser = purgeTarget.id;
    adminError = null;
    try {
      await purgeUserData(purgeTarget.id);
      purgeTarget = null;
      purgeConfirm = '';
      await loadAdmin();
    } catch (e) {
      adminError = (e as Error).message;
    } finally {
      busyUser = null;
    }
  }

  function rebindHostname(url: string): string {
    try {
      return new URL(url.trim()).hostname;
    } catch {
      return '';
    }
  }

  async function submitRebind(): Promise<void> {
    if (rebindBusy) return;
    rebindError = null;
    const host = rebindHostname(rebindUrl);
    if (!host) {
      rebindError = 'Enter a valid Immich URL.';
      return;
    }
    if (rebindConfirm.trim().toLowerCase() !== host.toLowerCase()) {
      rebindError = 'Type the new hostname to confirm.';
      return;
    }
    rebindBusy = true;
    try {
      await rebindInstance({
        immich_url: rebindUrl.trim(),
        confirm_hostname: rebindConfirm.trim(),
        email: rebindMethod === 'password' ? rebindEmail.trim() : undefined,
        password: rebindMethod === 'password' ? rebindPassword : undefined,
        api_key: rebindMethod === 'apikey' ? rebindApiKey.trim() : undefined
      });
      session.clear();
      window.location.replace('/login');
    } catch (e) {
      rebindError = (e as Error).message;
    } finally {
      rebindBusy = false;
    }
  }
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

    {#if session.user}
      <section class="space-y-2">
        <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Account</h2>
        <div class="flex items-center justify-between rounded bg-white/5 px-3 py-2">
          <div class="text-xs">
            <div class="font-medium">{session.user.name || session.user.email}</div>
            <div class="text-immich-dark-fg/50 font-mono">
              {session.user.email}
              {#if session.user.is_admin}<span class="text-immich-primary"> · admin</span>{/if}
              <span class="text-immich-dark-fg/40"> · {session.user.auth_kind === 'password' ? 'password' : 'API key'}</span>
            </div>
          </div>
          <button
            class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs disabled:opacity-50"
            onclick={() => void signOut()}
            disabled={signingOut}
          >
            {signingOut ? 'Signing out…' : 'Sign out'}
          </button>
        </div>
      </section>

      <section class="space-y-2">
        <div class="flex items-center justify-between">
          <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Active sessions</h2>
          {#if sessions.length > 1}
            <button
              class="px-2 py-1 rounded bg-white/5 hover:bg-white/10 text-xs"
              onclick={() => void revokeOthers()}
            >
              Revoke all other sessions
            </button>
          {/if}
        </div>
        {#if sessionsLoading}
          <Spinner label="Loading…" />
        {:else if sessions.length === 0}
          <p class="text-xs text-immich-dark-fg/50">No active sessions.</p>
        {:else}
          <ul class="space-y-1">
            {#each sessions as s (s.id)}
              <li class="flex items-center justify-between rounded bg-white/5 px-3 py-2 text-xs">
                <div class="min-w-0">
                  <div class="truncate">
                    {s.user_agent || 'Unknown device'}
                    {#if s.current}<span class="text-immich-primary"> · this session</span>{/if}
                  </div>
                  <div class="text-immich-dark-fg/40 font-mono">
                    {s.ip ?? '—'} · last seen {formatWhen(s.last_seen_at)}
                  </div>
                </div>
                {#if !s.current}
                  <button
                    class="ml-2 px-2 py-1 rounded bg-white/5 hover:bg-white/10 shrink-0"
                    onclick={() => void revokeOne(s.id)}
                  >
                    Revoke
                  </button>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      {#if session.isAdmin}
        <section class="space-y-2">
          <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Users</h2>
          {#if adminError}
            <p class="text-xs text-red-400">{adminError}</p>
          {/if}
          {#if usersLoading}
            <Spinner label="Loading…" />
          {:else if users.length === 0}
            <p class="text-xs text-immich-dark-fg/50">No users yet.</p>
          {:else}
            <ul class="space-y-1">
              {#each users as u (u.id)}
                <li class="flex items-center justify-between rounded bg-white/5 px-3 py-2 text-xs">
                  <div class="min-w-0">
                    <div class="truncate">
                      {u.name || u.email}
                      {#if u.is_admin}<span class="text-immich-primary"> · admin</span>{/if}
                      {#if !u.access_enabled}<span class="text-amber-300"> · disabled</span>{/if}
                    </div>
                    <div class="text-immich-dark-fg/40 font-mono truncate">{u.email}</div>
                  </div>
                  <div class="flex items-center gap-2 shrink-0 ml-2">
                    {#if u.id !== session.user?.id}
                      <button
                        class="px-2 py-1 rounded bg-white/5 hover:bg-white/10 disabled:opacity-50"
                        onclick={() => void toggleAccess(u)}
                        disabled={busyUser === u.id}
                      >
                        {u.access_enabled ? 'Disable' : 'Enable'}
                      </button>
                    {/if}
                    <button
                      class="px-2 py-1 rounded bg-red-500/10 text-red-300 hover:bg-red-500/20 disabled:opacity-50"
                      onclick={() => {
                        purgeTarget = u;
                        purgeConfirm = '';
                      }}
                      disabled={busyUser === u.id}
                    >
                      Purge data
                    </button>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}

          {#if purgeTarget}
            <div class="rounded border border-red-500/30 bg-red-500/5 p-3 space-y-2 text-xs">
              <p>
                Delete all edits, presets and export jobs for
                <span class="font-mono">{purgeTarget.email}</span>? This cannot be undone.
              </p>
              <p class="text-immich-dark-fg/50">
                Type <span class="font-mono">{purgeTarget.email}</span> to confirm.
              </p>
              <input
                class="w-full rounded bg-black/30 border border-white/10 px-2 py-1 font-mono"
                bind:value={purgeConfirm}
                placeholder={purgeTarget.email}
              />
              <div class="flex items-center gap-2">
                <button
                  class="px-3 py-1.5 rounded bg-red-500/20 text-red-200 hover:bg-red-500/30 disabled:opacity-50"
                  onclick={() => void confirmPurge()}
                  disabled={purgeConfirm !== purgeTarget.email || busyUser === purgeTarget.id}
                >
                  {busyUser === purgeTarget.id ? 'Purging…' : 'Purge data'}
                </button>
                <button
                  class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10"
                  onclick={() => {
                    purgeTarget = null;
                    purgeConfirm = '';
                  }}
                >
                  Cancel
                </button>
              </div>
            </div>
          {/if}
        </section>

        <section class="space-y-2">
          <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Mask models</h2>
          {#if modelsError}
            <p class="text-xs text-red-300">{modelsError}</p>
          {/if}
          {#if !models}
            <Spinner label="Loading…" />
          {:else if !models.enabled}
            <p class="text-xs text-immich-dark-fg/50">
              Segmentation is disabled. Set SEGMENT_RUNTIME to auto, gpu or cpu to enable it.
            </p>
          {:else}
            <p class="text-xs text-immich-dark-fg/50">
              Runtime: <span class="font-mono">{models.runtime}</span>. Models are downloaded on
              demand and loaded only while a mask is being generated.
            </p>
            <ul class="space-y-1">
              {#each models.models as m (m.id)}
                <li class="flex items-center justify-between rounded bg-white/5 px-3 py-2 text-xs">
                  <div class="min-w-0">
                    <div class="truncate">
                      {m.name}
                      <span class="text-immich-dark-fg/40"> · {m.kind}</span>
                      {#if m.tier === 'recommended'}<span class="text-immich-primary"> · recommended</span>{/if}
                    </div>
                    <div class="text-immich-dark-fg/40 truncate">
                      {formatMb(m.size_bytes)} · {m.gpu_mb} MB on GPU · {m.license}
                    </div>
                  </div>
                  <div class="flex items-center gap-2 shrink-0 ml-2">
                    <button
                      class="px-2 py-1 rounded disabled:opacity-50 {m.installed
                        ? 'bg-red-500/10 text-red-300 hover:bg-red-500/20'
                        : 'bg-white/5 hover:bg-white/10'}"
                      onclick={() => void toggleModel(m)}
                      disabled={busyModel === m.id}
                    >
                      {#if busyModel === m.id}
                        {m.installed ? 'Removing…' : 'Downloading…'}
                      {:else}
                        {m.installed ? 'Remove' : 'Download'}
                      {/if}
                    </button>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        </section>

        <section class="space-y-2">
          <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Immich instance</h2>
          {#if instance}
            <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs">
              <dt class="text-immich-dark-fg/50">Server URL</dt><dd class="font-mono break-all">{instance.immich_url}</dd>
              <dt class="text-immich-dark-fg/50">Config epoch</dt><dd class="font-mono">{instance.server_epoch}</dd>
              <dt class="text-immich-dark-fg/50">Configured</dt><dd class="font-mono">{formatWhen(instance.configured_at)}</dd>
            </dl>
          {/if}
          {#if !showRebind}
            <button
              class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs"
              onclick={() => (showRebind = true)}
            >
              Rebind to a different Immich server…
            </button>
          {:else}
            <div class="rounded border border-amber-500/30 bg-amber-500/5 p-3 space-y-2 text-xs">
              <p class="text-amber-200">
                Rebinding points immich-edit at a new Immich server and wipes all local users,
                edits and export jobs. Shared LUTs and camera profiles are kept. This cannot be
                undone.
              </p>
              <input
                class="w-full rounded bg-black/30 border border-white/10 px-2 py-1 font-mono"
                bind:value={rebindUrl}
                placeholder="https://immich.example.com"
              />
              <div class="flex items-center gap-2">
                <button
                  class="px-2 py-1 rounded {rebindMethod === 'password' ? 'bg-white/15' : 'bg-white/5 hover:bg-white/10'}"
                  onclick={() => (rebindMethod = 'password')}
                >
                  Password
                </button>
                <button
                  class="px-2 py-1 rounded {rebindMethod === 'apikey' ? 'bg-white/15' : 'bg-white/5 hover:bg-white/10'}"
                  onclick={() => (rebindMethod = 'apikey')}
                >
                  API key
                </button>
              </div>
              {#if rebindMethod === 'password'}
                <input
                  class="w-full rounded bg-black/30 border border-white/10 px-2 py-1"
                  bind:value={rebindEmail}
                  placeholder="admin@example.com"
                  autocomplete="off"
                />
                <input
                  class="w-full rounded bg-black/30 border border-white/10 px-2 py-1"
                  type="password"
                  bind:value={rebindPassword}
                  placeholder="Password"
                  autocomplete="off"
                />
              {:else}
                <input
                  class="w-full rounded bg-black/30 border border-white/10 px-2 py-1 font-mono"
                  bind:value={rebindApiKey}
                  placeholder="Immich API key"
                  autocomplete="off"
                />
              {/if}
              <p class="text-immich-dark-fg/50">
                Type the new hostname
                {#if rebindHostname(rebindUrl)}<span class="font-mono">{rebindHostname(rebindUrl)}</span>{/if}
                to confirm.
              </p>
              <input
                class="w-full rounded bg-black/30 border border-white/10 px-2 py-1 font-mono"
                bind:value={rebindConfirm}
                placeholder={rebindHostname(rebindUrl) || 'hostname'}
              />
              {#if rebindError}
                <p class="text-red-400">{rebindError}</p>
              {/if}
              <div class="flex items-center gap-2">
                <button
                  class="px-3 py-1.5 rounded bg-amber-500/20 text-amber-100 hover:bg-amber-500/30 disabled:opacity-50"
                  onclick={() => void submitRebind()}
                  disabled={rebindBusy}
                >
                  {rebindBusy ? 'Rebinding…' : 'Rebind and reset'}
                </button>
                <button
                  class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10"
                  onclick={() => {
                    showRebind = false;
                    rebindError = null;
                  }}
                >
                  Cancel
                </button>
              </div>
            </div>
          {/if}
        </section>
      {/if}
    {/if}

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
          <li><a class="text-immich-primary hover:underline" href="https://github.com/haavardnk/immich-edit/releases" target="_blank" rel="noopener">Releases</a></li>
          <li><a class="text-immich-primary hover:underline" href="https://github.com/haavardnk/immich-edit/issues/new/choose" target="_blank" rel="noopener">Report an issue</a></li>
        </ul>
      </section>
    {/if}
  </div>
</div>
