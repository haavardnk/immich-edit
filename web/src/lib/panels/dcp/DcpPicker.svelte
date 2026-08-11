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
    cameraModel,
    onSelect
  }: {
    profiles: DcpMeta[];
    mode: DcpMode;
    selectedId: string | null;
    autoMatch: DcpMeta | null;
    cameraModel: string | null;
    onSelect: (mode: DcpMode, id: string | null) => void;
  } = $props();

  let open = $state(false);
  let query = $state('');

  function profileLabel(profile: DcpMeta): string {
    return profile.camera_model ?? profile.name;
  }

  function matching(list: DcpMeta[], q: string): DcpMeta[] {
    const needle = q.trim().toLowerCase();
    if (!needle) return list;
    return list.filter(
      (profile) =>
        profile.name.toLowerCase().includes(needle) ||
        profile.camera_model?.toLowerCase().includes(needle) ||
        profile.copyright?.toLowerCase().includes(needle)
    );
  }

  const selected = $derived(profiles.find((profile) => profile.id === selectedId) ?? null);
  const importedProfiles = $derived(profiles.filter((profile) => !profile.bundled));
  const bundledProfiles = $derived(
    [...profiles.filter((profile) => profile.bundled)].sort((a, b) =>
      profileLabel(a).localeCompare(profileLabel(b))
    )
  );
  const filteredImported = $derived(matching(importedProfiles, query));
  const filteredBundled = $derived(matching(bundledProfiles, query));
  const showSearch = $derived(importedProfiles.length + bundledProfiles.length > 8);
  const autoDetail = $derived.by(() => {
    if (autoMatch) return profileLabel(autoMatch);
    if (!cameraModel) return 'Camera model unknown — using Default Color';
    return `No profile for ${cameraModel} — using Default Color`;
  });
  const label = $derived.by(() => {
    if (mode === 'off') return 'Default Color';
    if (mode === 'flat') return 'Flat';
    if (mode === 'auto') {
      return autoMatch ? `Auto · ${profileLabel(autoMatch)}` : 'Auto · Default Color';
    }
    return selected ? profileLabel(selected) : 'Profile missing';
  });

  function pick(nextMode: DcpMode, id: string | null): void {
    onSelect(nextMode, id);
    close();
  }

  function toggle(): void {
    open = !open;
    if (open && mode === 'auto' && !autoMatch && cameraModel) {
      query = cameraModel;
    }
  }

  function close(): void {
    open = false;
    query = '';
  }
</script>

{#snippet group(title: string, list: DcpMeta[])}
  <div class="px-3 pb-0.5 pt-1.5 text-[10px] uppercase tracking-wider text-immich-dark-fg/40">
    {title}
  </div>
  {#each list as profile (profile.id)}
    <button
      type="button"
      class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors {mode ===
        'profile' && selectedId === profile.id
        ? 'bg-immich-dark-primary/20 text-immich-dark-primary'
        : 'hover:bg-white/10'}"
      title={profileLabel(profile)}
      onclick={() => pick('profile', profile.id)}
    >
      <span class="w-3.5 shrink-0">
        {#if mode === 'profile' && selectedId === profile.id}<Icon path={mdiCheck} size={14} />{/if}
      </span>
      <span class="truncate">{profileLabel(profile)}</span>
    </button>
  {/each}
{/snippet}

<Popover
  {open}
  onClose={close}
  rootClass="w-full"
  contentClass="w-full max-h-80 overflow-hidden flex flex-col"
>
  {#snippet trigger()}
    <button
      type="button"
      class="flex w-full items-center gap-2 rounded-lg bg-white/5 px-2.5 py-1.5 text-xs transition-colors hover:bg-white/10"
      title={label}
      onclick={toggle}
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
      <span class="w-3.5 shrink-0"
        >{#if mode === 'auto'}<Icon path={mdiCheck} size={14} />{/if}</span
      >
      <span class="min-w-0 flex-1">
        <span class="block truncate">Auto match</span>
        <span class="block truncate text-[10px] opacity-60">{autoDetail}</span>
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
      <span class="w-3.5 shrink-0"
        >{#if mode === 'off'}<Icon path={mdiCheck} size={14} />{/if}</span
      >
      <span class="min-w-0 flex-1">
        <span class="block truncate">Default Color</span>
        <span class="block truncate text-[10px] opacity-60">Our general-purpose rendering</span>
      </span>
    </button>
    <button
      type="button"
      class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors {mode ===
      'flat'
        ? 'bg-immich-dark-primary/20 text-immich-dark-primary'
        : 'hover:bg-white/10'}"
      onclick={() => pick('flat', null)}
    >
      <span class="w-3.5 shrink-0"
        >{#if mode === 'flat'}<Icon path={mdiCheck} size={14} />{/if}</span
      >
      <span class="min-w-0 flex-1">
        <span class="block truncate">Flat</span>
        <span class="block truncate text-[10px] opacity-60">No tone curve, bare sensor colour</span>
      </span>
    </button>
  </div>

  {#if showSearch}
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
    {#if filteredImported.length === 0 && filteredBundled.length === 0}
      <div class="px-3 py-2 text-xs text-immich-dark-fg/30">No matching profiles.</div>
    {:else}
      {#if filteredImported.length > 0}
        {@render group('Imported profiles', filteredImported)}
      {/if}
      {#if filteredBundled.length > 0}
        {@render group('Camera profiles', filteredBundled)}
      {/if}
    {/if}
  </div>
</Popover>
