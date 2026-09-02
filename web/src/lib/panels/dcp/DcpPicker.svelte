<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import type { DcpMeta } from '$lib/api/dcp';
  import type { DcpMode } from '$lib/types/edits';
  import { mergeProps } from '$lib/utils/mergeProps';
  import { Button, Icon, Tooltip } from '@immich/ui';
  import { mdiCheck, mdiChevronDown } from '@mdi/js';

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

  function profileLabel(profile: DcpMeta): string {
    return profile.camera_model ?? profile.name;
  }

  const selected = $derived(profiles.find((profile) => profile.id === selectedId) ?? null);
  const sortedProfiles = $derived(
    [...profiles].sort((left, right) => {
      if (left.bundled !== right.bundled) return left.bundled ? 1 : -1;
      return profileLabel(left).localeCompare(profileLabel(right));
    })
  );
  const autoDetail = $derived.by(() => {
    if (autoMatch) return profileLabel(autoMatch);
    if (!cameraModel) return 'Camera model unknown, using Default Color';
    return `No profile for ${cameraModel}, using Default Color`;
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
    open = false;
  }

  function toggle(next: boolean): void {
    open = next;
  }

  function selectProfile(next: string[]): void {
    const id = next.at(-1);
    if (!id) {
      pick('auto', null);
      return;
    }
    pick('profile', id);
  }
</script>

{#snippet check(active: boolean)}
  <span class="flex w-4 shrink-0 justify-center text-primary">
    {#if active}<Icon icon={mdiCheck} size="14px" aria-hidden="true" />{/if}
  </span>
{/snippet}

<Popover
  {open}
  onOpenChange={toggle}
  contentClass="w-(--bits-floating-anchor-width) max-h-80 overflow-hidden flex flex-col rounded-lg bg-neutral-900 p-1"
>
  {#snippet trigger(popoverProps)}
    <Tooltip text={label}>
      {#snippet child({ props })}
        <Button
          type="button"
          size="tiny"
          variant="ghost"
          color="secondary"
          fullWidth
          class="h-7 justify-start rounded-lg bg-neutral-800 px-2 text-[11px] hover:bg-neutral-700"
          {...mergeProps(props, popoverProps)}
        >
          <span class="flex-1 truncate text-left">{label}</span>
          <Icon
            icon={mdiChevronDown}
            size="14px"
            class="shrink-0 opacity-45 transition-transform {open ? 'rotate-180' : ''}"
            aria-hidden="true"
          />
        </Button>
      {/snippet}
    </Tooltip>
  {/snippet}

  <div class="border-b border-hairline pb-1">
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      fullWidth
      class="min-h-9 justify-start rounded-md px-2 py-1 {mode === 'auto'
        ? 'bg-primary/12 text-white'
        : 'text-white/80 hover:bg-white/6'}"
      onclick={() => pick('auto', null)}
    >
      {@render check(mode === 'auto')}
      <span class="min-w-0 flex-1 text-left">
        <span class="block truncate">Auto match</span>
        <span class="block truncate text-[10px] text-white/65">{autoDetail}</span>
      </span>
    </Button>
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      fullWidth
      class="min-h-9 justify-start rounded-md px-2 py-1 {mode === 'off'
        ? 'bg-primary/12 text-white'
        : 'text-white/80 hover:bg-white/6'}"
      onclick={() => pick('off', null)}
    >
      {@render check(mode === 'off')}
      <span class="min-w-0 flex-1 text-left">
        <span class="block truncate">Default Color</span>
        <span class="block truncate text-[10px] text-white/65">General-purpose rendering</span>
      </span>
    </Button>
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      fullWidth
      class="min-h-9 justify-start rounded-md px-2 py-1 {mode === 'flat'
        ? 'bg-primary/12 text-white'
        : 'text-white/80 hover:bg-white/6'}"
      onclick={() => pick('flat', null)}
    >
      {@render check(mode === 'flat')}
      <span class="min-w-0 flex-1 text-left">
        <span class="block truncate">Flat</span>
        <span class="block truncate text-[10px] text-white/65">
          No tone curve, bare sensor colour
        </span>
      </span>
    </Button>
  </div>

  <div class="pt-1">
    <SearchableSelect
      compact
      multiple={false}
      color="neutral"
      options={sortedProfiles}
      selected={mode === 'profile' && selectedId ? [selectedId] : []}
      getId={(profile) => profile.id}
      getLabel={profileLabel}
      getGroup={(profile) => (profile.bundled ? null : 'Imported profiles')}
      getSearchText={(profile) =>
        `${profile.name} ${profile.camera_model ?? ''} ${profile.copyright ?? ''}`}
      placeholder="Search camera profiles"
      hideSelected={false}
      onSelectedChange={selectProfile}
    />
  </div>
</Popover>
