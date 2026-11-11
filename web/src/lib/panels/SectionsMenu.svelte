<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import SectionList from '#lib/components/editor/sections/SectionList.svelte';
  import type { SectionGroup } from '#lib/components/editor/sections/sectionList.ts';
  import { DESKTOP_PANELS } from '#lib/editor/params/catalog.ts';
  import { desktopSections } from '#lib/stores/developSections.svelte.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiAccountOutline, mdiDotsHorizontal } from '@mdi/js';

  const LOCKED = [
    { label: 'Scopes', note: 'always first' },
    { label: 'Quick', note: 'pinned settings' }
  ];

  let open = $state(false);

  const groups = $derived<SectionGroup[]>([
    {
      id: 'develop',
      rows: desktopSections.entries.map((entry) => ({
        id: entry.id,
        label: DESKTOP_PANELS.find((panel) => panel.id === entry.id)?.title ?? entry.id,
        hidden: entry.hidden
      }))
    }
  ]);
</script>

<Popover
  {open}
  onOpenChange={(next) => (open = next)}
  anchor="bottom"
  align="end"
  contentClass="w-72 p-1.5"
  tooltip="Customize sections"
>
  {#snippet trigger(props)}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="bg-transparent hover:bg-light-950/6"
      icon={mdiDotsHorizontal}
      title=""
      aria-label="Customize sections"
      {...props}
    />
  {/snippet}
  <div class="px-1.5 pt-1 pb-1.5">
    <p class="text-[10px] font-medium tracking-wider text-dark/65 uppercase">Customize sections</p>
    <p class="text-[11px] text-dark/55">Drag to reorder · the eye hides a section</p>
  </div>
  <SectionList
    {groups}
    locked={LOCKED}
    variant="menu"
    onreorder={(ids) => void desktopSections.reorder(ids)}
    ontoggle={(id) => void desktopSections.toggle(id)}
  />
  <div class="mt-1 flex items-center gap-2 border-t border-dark/10 px-1.5 pt-1.5">
    <span class="flex min-w-0 flex-1 items-center gap-1 text-[11px] text-dark/55">
      <Icon icon={mdiAccountOutline} size="13px" class="shrink-0" aria-hidden="true" />
      Saved to your account
    </span>
    <Button
      size="tiny"
      variant="ghost"
      color="primary"
      class="shrink-0"
      disabled={!desktopSections.customised}
      onclick={() => void desktopSections.reset()}
    >
      Reset to default
    </Button>
  </div>
</Popover>
