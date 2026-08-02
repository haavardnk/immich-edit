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
  import {
    listMaskModels,
    installMaskModel,
    removeMaskModel,
    selectMaskModel,
    type MaskKind,
    type MaskModel,
    type MaskModelsResponse
  } from '$lib/api/masks';
  import Spinner from '$lib/components/Spinner.svelte';

  onMount(() => {
    editor.unload();
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
  let busyModels = $state<string[]>([]);

  const modelKinds = $derived([...new Set((models?.models ?? []).map((m) => m.kind))]);
  const missingRecommended = $derived(
    (models?.models ?? []).filter((m) => !m.installed && m.tier === 'recommended')
  );

  let expandedKinds = $state<string[]>([]);

  function toggleKind(kind: string): void {
    expandedKinds = expandedKinds.includes(kind)
      ? expandedKinds.filter((k) => k !== kind)
      : [...expandedKinds, kind];
  }

  async function loadModels(): Promise<void> {
    try {
      models = await listMaskModels();
    } catch (e) {
      modelsError = (e as Error).message;
    }
  }

  async function toggleModel(m: MaskModel): Promise<void> {
    if (busyModels.includes(m.id)) return;
    busyModels = [...busyModels, m.id];
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
      busyModels = busyModels.filter((id) => id !== m.id);
    }
  }

  async function downloadRecommended(): Promise<void> {
    modelsError = null;
    await Promise.all(missingRecommended.map((m) => toggleModel(m)));
  }

  async function chooseModel(kind: MaskKind, modelId: string): Promise<void> {
    modelsError = null;
    try {
      await selectMaskModel(kind, modelId);
      await loadModels();
    } catch (e) {
      modelsError = (e as Error).message;
    }
  }

  function kindLabel(kind: string): string {
    if (kind === 'subject') return 'Subject';
    if (kind === 'people') return 'People';
    if (kind === 'sky') return 'Sky';
    if (kind === 'depth') return 'Depth';
    if (kind === 'semantic') return 'Scene classes';
    if (kind === 'click') return 'Click to select';
    return kind;
  }

  function formatSeconds(ms: number): string {
    return ms >= 1000 ? `${(ms / 1000).toFixed(1)} s` : `${ms} ms`;
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
                <div class="min-w-0 flex-1">
                  <div class="truncate">
                    {s.user_agent || 'Unknown device'}
                    {#if s.current}<span class="text-immich-primary"> · this session</span>{/if}
                  </div>
                  <div class="text-immich-dark-fg/40 font-mono truncate">
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

            {#if missingRecommended.length > 0}
              <button
                class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs disabled:opacity-50"
                onclick={() => void downloadRecommended()}
                disabled={busyModels.length > 0}
              >
                Download {missingRecommended.length} recommended ({formatMb(
                  missingRecommended.reduce((n, m) => n + m.size_bytes, 0)
                )})
              </button>
            {/if}

            {#snippet modelRow(m: MaskModel)}
              <li class="flex items-center justify-between rounded bg-white/5 px-3 py-2 text-xs">
                <div class="min-w-0 flex-1">
                  <div class="truncate">
                    {m.name}
                    {#if m.tier === 'recommended'}<span class="text-immich-primary"> · recommended</span>{/if}
                    <span class="text-immich-dark-fg/40"> · {m.license}</span>
                  </div>
                  <div class="text-immich-dark-fg/40 truncate">{m.notes}</div>
                  <div class="text-immich-dark-fg/40 truncate">
                    {formatMb(m.size_bytes)} download · ~{m.gpu_mb} MB while running · ~{formatSeconds(
                      m.gpu_ms
                    )} per photo on GPU{#if m.cpu_ms > 0}, ~{formatSeconds(m.cpu_ms)} on CPU{/if}
                  </div>
                </div>
                <div class="flex items-center gap-2 shrink-0 ml-2">
                  <button
                    class="px-2 py-1 rounded disabled:opacity-50 {m.installed
                      ? 'bg-red-500/10 text-red-300 hover:bg-red-500/20'
                      : 'bg-white/5 hover:bg-white/10'}"
                    onclick={() => void toggleModel(m)}
                    disabled={busyModels.includes(m.id)}
                  >
                    {#if busyModels.includes(m.id)}
                      {m.installed ? 'Removing…' : 'Downloading…'}
                    {:else}
                      {m.installed ? 'Remove' : 'Download'}
                    {/if}
                  </button>
                </div>
              </li>
            {/snippet}

            <div class="space-y-3">
              {#each modelKinds as kind (kind)}
                {@const inKind = models.models.filter((m) => m.kind === kind)}
                {@const installed = inKind.filter((m) => m.installed)}
                {@const primary = installed.length > 0 ? installed : inKind.filter((m) => m.tier === 'recommended')}
                {@const alternatives = inKind.filter((m) => !primary.includes(m))}
                {@const open = expandedKinds.includes(kind)}
                <div class="space-y-1">
                  <div class="flex items-center justify-between gap-2 text-xs">
                    <span class="text-immich-dark-fg/50">{kindLabel(kind)} masks use</span>
                    {#if installed.length === 0}
                      <span class="text-immich-dark-fg/40">nothing installed</span>
                    {:else}
                      <select
                        class="rounded bg-black/30 border border-white/10 px-2 py-1"
                        value={models.active[kind] ?? installed[0].id}
                        onchange={(e) => void chooseModel(kind, e.currentTarget.value)}
                      >
                        {#each installed as m (m.id)}
                          <option value={m.id}>{m.name}</option>
                        {/each}
                      </select>
                    {/if}
                  </div>
                  <ul class="space-y-1">
                    {#each primary as m (m.id)}
                      {@render modelRow(m)}
                    {/each}
                    {#if open}
                      {#each alternatives as m (m.id)}
                        {@render modelRow(m)}
                      {/each}
                    {/if}
                  </ul>
                  {#if alternatives.length > 0}
                    <button
                      class="text-xs text-immich-dark-fg/50 hover:text-immich-dark-fg/80"
                      onclick={() => toggleKind(kind)}
                    >
                      {open ? 'Hide' : 'Show'} {alternatives.length} alternative{alternatives.length === 1 ? '' : 's'}
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
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
