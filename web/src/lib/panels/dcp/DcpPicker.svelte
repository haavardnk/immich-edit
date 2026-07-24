<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import type { DcpMeta } from '$lib/api/dcp';
  import type { DcpMode } from '$lib/types/edits';
  import { mdiCheck, mdiChevronDown, mdiMagnify } from '@mdi/js';

  let {
    profiles,
    mode,
    selectedId,
    autoMatch,
    onSelect
  }: {
    profiles: DcpMeta[];
    mode: DcpMode;
    selectedId: string | null;
    autoMatch: DcpMeta | null;
    onSelect: (mode: DcpMode, id: string | null) => void;
  } = $props();

  let open = $state(false);
  let query = $state('');

  const selected = $derived(profiles.find((profile) => profile.id === selectedId) ?? null);
  const importedProfiles = $derived(profiles.filter((profile) => !profile.bundled));
  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return importedProfiles;
    return importedProfiles.filter(
      (profile) =>
        profile.name.toLowerCase().includes(q) ||
        profile.camera_model?.toLowerCase().includes(q) ||
        profile.copyright?.toLowerCase().includes(q)
    );
  });
  const label = $derived.by(() => {
    if (mode === 'off') return 'Default Color';
    if (mode === 'auto') {
      const matched = autoMatch?.camera_model ?? autoMatch?.name;
      return matched ? `Auto · ${matched}` : 'Auto · No camera match';
    }
    return selected?.camera_model ?? selected?.name ?? 'Unavailable profile';
  });

  function pick(nextMode: DcpMode, id: string | null): void {
    onSelect(nextMode, id);
    close();
  }

  function close(): void {
    open = false;
    query = '';
  }
</script>

<Popover {open} onClose={close} rootClass="w-full" contentClass="w-full max-h-80 overflow-hidden flex flex-col">
  {#snippet trigger()}
    <button
      type="button"
      class="flex w-full items-center gap-2 rounded-lg bg-white/5 px-2.5 py-1.5 text-xs transition-colors hover:bg-white/10"
      title={label}
      onclick={() => (open = !open)}
    >
      <span class="flex-1 truncate text-left">{label}</span>
      <Icon path={mdiChevronDown} size={14} class="flex-none opacity-50" />
    </button>
  {/snippet}

  <div class="border-b border-white/10 py-1">
    <button
      type="button"
      class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors {mode ===
      'auto'
        ? 'bg-immich-dark-primary/20 text-immich-dark-primary'
        : 'hover:bg-white/10'}"
      onclick={() => pick('auto', null)}
    >
      <span class="w-3.5 shrink-0">{#if mode === 'auto'}<Icon path={mdiCheck} size={14} />{/if}</span>
      <span class="min-w-0 flex-1">
        <span class="block truncate">Auto match</span>
        <span class="block truncate text-[10px] opacity-60">
          {autoMatch?.camera_model ?? autoMatch?.name ?? 'No matching bundled profile'}
        </span>
      </span>
    </button>
    <button
      type="button"
      class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors {mode ===
      'off'
        ? 'bg-immich-dark-primary/20 text-immich-dark-primary'
        : 'hover:bg-white/10'}"
      onclick={() => pick('off', null)}
    >
      <span class="w-3.5 shrink-0">{#if mode === 'off'}<Icon path={mdiCheck} size={14} />{/if}</span>
      <span>Default Color</span>
    </button>
  </div>

  {#if importedProfiles.length > 8}
    <div class="border-b border-white/10 p-1.5">
      <div class="flex items-center gap-1.5 rounded-lg bg-white/5 px-2 py-1">
        <Icon path={mdiMagnify} size={14} class="flex-none opacity-40" />
        <input
          class="min-w-0 flex-1 bg-transparent text-xs outline-none placeholder:text-immich-dark-fg/30"
          placeholder="Search camera profiles"
          bind:value={query}
        />
      </div>
    </div>
  {/if}

  <div class="overflow-y-auto py-1">
    {#if importedProfiles.length === 0}
      <div class="px-3 py-2 text-xs text-immich-dark-fg/30">No imported profiles.</div>
    {:else if filtered.length === 0}
      <div class="px-3 py-2 text-xs text-immich-dark-fg/30">No matching imported profiles.</div>
    {:else}
      <div class="px-3 pb-0.5 pt-1.5 text-[10px] uppercase tracking-wider text-immich-dark-fg/40">
        Imported profiles
      </div>
      {#each filtered as profile (profile.id)}
        <button
          type="button"
          class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors {mode ===
          'profile' && selectedId === profile.id
            ? 'bg-immich-dark-primary/20 text-immich-dark-primary'
            : 'hover:bg-white/10'}"
          title={profile.camera_model ?? profile.name}
          onclick={() => pick('profile', profile.id)}
        >
          <span class="w-3.5 shrink-0">
            {#if mode === 'profile' && selectedId === profile.id}<Icon path={mdiCheck} size={14} />{/if}
          </span>
          <span class="truncate">{profile.camera_model ?? profile.name}</span>
        </button>
      {/each}
    {/if}
  </div>
</Popover>
