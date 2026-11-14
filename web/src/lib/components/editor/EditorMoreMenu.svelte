<script lang="ts">
  import { holdOriginal } from '#lib/commands/editor/view.ts';
  import { EDITOR_MORE_MENU, EDITOR_VIEW_MENU } from '#lib/commands/menus.ts';
  import CommandList from '#lib/components/CommandList.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { IconButton } from '@immich/ui';
  import {
    mdiCameraOutline,
    mdiContentDuplicate,
    mdiDotsVertical,
    mdiHistory,
    mdiViewSequentialOutline
  } from '@mdi/js';
  import SoftProofControl from './SoftProofControl.svelte';
  import HistorySheet from './history/HistorySheet.svelte';
  import EditRows from './more/EditRows.svelte';
  import SheetRow from './more/SheetRow.svelte';
  import SnapshotsSheet from './snapshots/SnapshotsSheet.svelte';
  import VersionsList from './versions/VersionsList.svelte';
  import VersionsSheet from './versions/VersionsSheet.svelte';
  import SectionsSheet from './touch/SectionsSheet.svelte';
  import { touchSections } from '#lib/stores/developSections.svelte.ts';

  type Sheet = 'history' | 'snapshots' | 'versions' | 'sections';

  let { icon = mdiDotsVertical }: { icon?: string } = $props();

  let open = $state(false);
  let sheet = $state<Sheet | null>(null);
  let after: (() => void) | null = null;

  const touch = $derived(layout.mode === 'touch');

  function setOpen(next: boolean): void {
    open = next;
    if (next) {
      if (touch) {
        void editor.historySteps.refresh();
        void editor.versions.load();
      }
      return;
    }
    const action = after;
    after = null;
    action?.();
  }

  function then(action: () => void): void {
    after = action;
  }

  function show(name: Sheet): void {
    then(() => (sheet = name));
  }

  function sheetChange(name: Sheet, next: boolean): void {
    if (next) sheet = name;
    else if (sheet === name) sheet = null;
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
      color={editor.isProofing || ui.clipWarn || editor.compare.mode !== 'off'
        ? 'primary'
        : 'secondary'}
      {icon}
      title=""
      aria-label="More editor actions"
      {...props}
    />
  {/snippet}
  <div class="flex flex-col">
    <SheetRow
      icon={mdiHistory}
      label="History"
      detail={countOf(editor.historySteps.entries.length, 'step')}
      disabled={!editor.initialised}
      onclick={() => show('history')}
    />
    <SheetRow
      icon={mdiCameraOutline}
      label="Snapshots"
      detail={String(editor.snapshots.items.length)}
      disabled={!editor.initialised}
      onclick={() => show('snapshots')}
    />
    <SheetRow
      icon={mdiContentDuplicate}
      label="Versions and copies"
      detail={countOf(editor.versions.copies.length, 'copy', 'copies')}
      disabled={!editor.initialised}
      onclick={() => show('versions')}
    />
    {#if !touch}
      <VersionsList onnavigate={() => (open = false)} />
    {/if}
    <div class="my-1 border-t border-dark/10"></div>
    <EditRows {then} />
    <div class="my-1 hidden border-t border-dark/10 touch:block"></div>
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
    <SheetRow
      icon={mdiViewSequentialOutline}
      label="Customize Develop"
      detail={touchSections.hiddenCount > 0 ? `${touchSections.hiddenCount} hidden` : ''}
      onclick={() => show('sections')}
    />
  </div>
</Popover>

<HistorySheet open={sheet === 'history'} onOpenChange={(next) => sheetChange('history', next)} />
<SnapshotsSheet
  open={sheet === 'snapshots'}
  onOpenChange={(next) => sheetChange('snapshots', next)}
/>
<VersionsSheet open={sheet === 'versions'} onOpenChange={(next) => sheetChange('versions', next)} />
<SectionsSheet open={sheet === 'sections'} onOpenChange={(next) => sheetChange('sections', next)} />
