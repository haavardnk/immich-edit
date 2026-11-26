<script lang="ts">
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import BulkEditsMenu from '#lib/components/browse/bulk/BulkEditsMenu.svelte';
  import BulkMetadataMenu from '#lib/components/browse/bulk/BulkMetadataMenu.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { ROOMY_ONLY, TIGHT_ONLY } from '#lib/browse/bulk/bulkBarFit.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { Icon, IconButton } from '@immich/ui';
  import {
    mdiChevronRight,
    mdiContentDuplicate,
    mdiDotsHorizontal,
    mdiFileDocumentEditOutline,
    mdiImageAlbum,
    mdiTagOutline,
    mdiTuneVariant
  } from '@mdi/js';

  type Band = 'tags' | 'albums';

  type Tool = {
    label: string;
    tooltip: string;
    icon: string;
    ownerOnly: boolean;
    pressed?: boolean;
    disabled: boolean;
    run: () => void;
  };

  const EDITS_TOOLTIP = 'Apply a preset, paste or reset edits';

  let {
    band,
    onBand
  }: {
    band: Band | null;
    onBand: (band: Band | null) => void;
  } = $props();

  let menu = $state(false);
  let edits = $state<'bar' | 'menu' | null>(null);
  let metadata = $state<'bar' | 'menu' | null>(null);

  const scope = bulkScope;
  const actions = bulkActions;
  const shared = $derived(!scope.allOwned);

  const copyTool: Tool = $derived({
    label: 'Create virtual copy',
    tooltip: hint(
      scope.targetCount === 1 ? 'Create a virtual copy' : 'Create a virtual copy of each',
      'createVirtualCopy'
    ),
    icon: mdiContentDuplicate,
    ownerOnly: false,
    disabled: actions.metaBusy,
    run: () => void actions.createCopies()
  });

  const tagsTool: Tool = $derived({
    label: 'Tags',
    tooltip: 'Tags',
    icon: mdiTagOutline,
    ownerOnly: true,
    pressed: band === 'tags',
    disabled: actions.metaBusy,
    run: () => onBand(band === 'tags' ? null : 'tags')
  });

  const albumsTool: Tool = $derived({
    label: 'Albums',
    tooltip: 'Albums',
    icon: mdiImageAlbum,
    ownerOnly: false,
    pressed: band === 'albums',
    disabled: actions.metaBusy,
    run: () => onBand(band === 'albums' ? null : 'albums')
  });

  function openMetadata(where: 'bar' | 'menu', open: boolean): void {
    if (open && !actions.ownedOrExplain()) return;
    metadata = open ? where : null;
  }

  function pick(tool: Tool): void {
    menu = false;
    if (tool.ownerOnly && !actions.ownedOrExplain()) return;
    tool.run();
  }

  function setMenu(open: boolean): void {
    menu = open;
    if (!open && edits === 'menu') edits = null;
    if (!open && metadata === 'menu') metadata = null;
  }
</script>

{#snippet button(tool: Tool)}
  <IconButton
    size="medium"
    variant={tool.pressed ? 'filled' : 'ghost'}
    color={tool.pressed ? 'primary' : 'secondary'}
    icon={tool.icon}
    title={tool.tooltip}
    aria-label={tool.label}
    aria-pressed={tool.pressed}
    disabled={tool.disabled || (tool.ownerOnly && shared)}
    onclick={tool.run}
  />
{/snippet}

{#snippet barTool(tool: Tool)}
  {#if tool.ownerOnly}
    <DisabledReason locked={shared} reason={scope.notOwnedReason} class="flex">
      {@render button(tool)}
    </DisabledReason>
  {:else}
    {@render button(tool)}
  {/if}
{/snippet}

{#snippet row(tool: Tool)}
  <MenuRow
    icon={tool.icon}
    label={tool.label}
    pressed={tool.pressed}
    class={tool.ownerOnly && shared ? 'opacity-50' : ''}
    aria-label={tool.label}
    aria-disabled={(tool.ownerOnly && shared) || undefined}
    disabled={tool.disabled}
    onclick={() => pick(tool)}
  />
{/snippet}

<div class="{ROOMY_ONLY} shrink-0 items-center gap-1">
  {@render barTool(copyTool)}
  <BulkEditsMenu
    open={edits === 'bar'}
    onOpenChange={(open) => (edits = open ? 'bar' : null)}
    tooltip={EDITS_TOOLTIP}
  >
    {#snippet trigger(props)}
      <IconButton
        {...props}
        size="medium"
        variant={edits === 'bar' ? 'filled' : 'ghost'}
        color={edits === 'bar' ? 'primary' : 'secondary'}
        icon={mdiTuneVariant}
        title=""
        aria-label="Edits"
        disabled={scope.selectingAll}
      />
    {/snippet}
  </BulkEditsMenu>
  {@render barTool(tagsTool)}
  {@render barTool(albumsTool)}
  <DisabledReason locked={shared} reason={scope.notOwnedReason} class="flex">
    <BulkMetadataMenu
      open={metadata === 'bar'}
      onOpenChange={(open) => openMetadata('bar', open)}
      tooltip="Metadata"
    >
      {#snippet trigger(props)}
        <IconButton
          {...props}
          size="medium"
          variant={metadata === 'bar' ? 'filled' : 'ghost'}
          color={metadata === 'bar' ? 'primary' : 'secondary'}
          icon={mdiFileDocumentEditOutline}
          title=""
          aria-label="Metadata"
          disabled={actions.metaBusy || shared}
        />
      {/snippet}
    </BulkMetadataMenu>
  </DisabledReason>
</div>
<Popover
  open={menu}
  anchor="top"
  align="end"
  onOpenChange={setMenu}
  contentClass="w-60 py-1"
  tooltip="More actions"
>
  {#snippet trigger(popoverProps)}
    <IconButton
      {...popoverProps}
      size="medium"
      variant={band ? 'filled' : 'ghost'}
      color={band ? 'primary' : 'secondary'}
      class="shrink-0 {TIGHT_ONLY}"
      icon={mdiDotsHorizontal}
      title=""
      aria-label="More actions"
    />
  {/snippet}
  <div class="flex flex-col">
    {@render row(copyTool)}
    <BulkEditsMenu
      open={edits === 'menu'}
      anchor="right"
      onOpenChange={(open) => (edits = open ? 'menu' : null)}
      onDone={() => (menu = false)}
    >
      {#snippet trigger(props)}
        <MenuRow
          {...props}
          icon={mdiTuneVariant}
          label="Edits"
          aria-label="Edits"
          pressed={edits === 'menu'}
          disabled={scope.selectingAll}
        >
          <Icon icon={mdiChevronRight} size="14px" class="shrink-0 opacity-70" aria-hidden="true" />
        </MenuRow>
      {/snippet}
    </BulkEditsMenu>
    {@render row(tagsTool)}
    {@render row(albumsTool)}
    <BulkMetadataMenu
      open={metadata === 'menu'}
      anchor="right"
      onOpenChange={(open) => openMetadata('menu', open)}
      onDone={() => (menu = false)}
    >
      {#snippet trigger(props)}
        <MenuRow
          {...props}
          icon={mdiFileDocumentEditOutline}
          label="Metadata"
          aria-label="Metadata"
          class={shared ? 'opacity-50' : ''}
          aria-disabled={shared || undefined}
          pressed={metadata === 'menu'}
          disabled={actions.metaBusy}
        >
          <Icon icon={mdiChevronRight} size="14px" class="shrink-0 opacity-70" aria-hidden="true" />
        </MenuRow>
      {/snippet}
    </BulkMetadataMenu>
  </div>
</Popover>
