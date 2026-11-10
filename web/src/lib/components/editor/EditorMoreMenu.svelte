<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import SoftProofControl from './SoftProofControl.svelte';
  import VersionsList from './VersionsList.svelte';
  import SnapshotsSheet from './snapshots/SnapshotsSheet.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import CommandList from '#lib/components/CommandList.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import { holdOriginal } from '#lib/commands/editor/view.ts';
  import { EDITOR_MORE_MENU, EDITOR_VIEW_MENU } from '#lib/commands/menus.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiCameraOutline, mdiChevronRight, mdiDotsVertical } from '@mdi/js';

  let { icon = mdiDotsVertical }: { icon?: string } = $props();

  let open = $state(false);
  let snapshotsOpen = $state(false);
  let snapshotsNext = false;

  function setOpen(next: boolean): void {
    open = next;
    if (next || !snapshotsNext) return;
    snapshotsNext = false;
    snapshotsOpen = true;
  }
</script>

<Popover
  {open}
  anchor="bottom"
  align="end"
  onOpenChange={setOpen}
  contentClass="w-64 py-1"
  tooltip="More editor actions"
>
  {#snippet trigger(props)}
    <IconButton
      size="small"
      variant="ghost"
      color={editor.isProofing || ui.clipWarn || editor.splitMode ? 'primary' : 'secondary'}
      {icon}
      title=""
      aria-label="More editor actions"
      {...props}
    />
  {/snippet}
  <div class="flex flex-col">
    <MenuRow
      icon={mdiCameraOutline}
      label="Snapshots"
      class="hidden touch:flex"
      aria-label="Snapshots"
      disabled={!editor.initialised}
      onclick={() => (snapshotsNext = true)}
    >
      <span class="shrink-0 text-[13px] text-dark/65">{editor.snapshots.items.length}</span>
      <Icon icon={mdiChevronRight} size="20px" class="shrink-0 text-dark/65" aria-hidden="true" />
    </MenuRow>
    <VersionsList onnavigate={() => (open = false)} />
    <div class="my-1 border-t border-dark/10"></div>
    <span class="px-3 pt-1.5 pb-1 text-[10px] tracking-wider text-dark/65 uppercase">View</span>
    <CommandList
      menu={EDITOR_VIEW_MENU}
      onrun={() => (open = false)}
      rowClass={(command) => (command === holdOriginal ? 'xl:hidden touch:hidden' : undefined)}
    />
    <div class="my-1 border-t border-dark/10"></div>
    <SoftProofControl />
    <div class="my-1 border-t border-dark/10"></div>
    <CommandList menu={EDITOR_MORE_MENU} onrun={() => (open = false)} />
  </div>
</Popover>

<SnapshotsSheet open={snapshotsOpen} onOpenChange={(next) => (snapshotsOpen = next)} />
