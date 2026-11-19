<script lang="ts">
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { ROOMY_ONLY, TIGHT_ONLY } from '#lib/browse/bulk/bulkBarFit.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';
  import {
    mdiContentDuplicate,
    mdiDotsHorizontal,
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

  let {
    band,
    onBand,
    onEditExport
  }: {
    band: Band | null;
    onBand: (band: Band | null) => void;
    onEditExport: () => void;
  } = $props();

  let menu = $state(false);

  const scope = bulkScope;
  const actions = bulkActions;
  const shared = $derived(!scope.allOwned);
  const photos = $derived(scope.targetCount === 1 ? 'this photo' : 'selected photos');

  const tools: Tool[] = $derived([
    {
      label: 'Create virtual copy',
      tooltip: hint(
        scope.targetCount === 1 ? 'Create a virtual copy' : 'Create a virtual copy of each',
        'createVirtualCopy'
      ),
      icon: mdiContentDuplicate,
      ownerOnly: false,
      disabled: actions.metaBusy,
      run: () => void actions.createCopies()
    },
    {
      label: 'Edit and export selected',
      tooltip: `Paste edits, apply a preset or export ${photos}`,
      icon: mdiTuneVariant,
      ownerOnly: false,
      disabled: scope.selectingAll,
      run: onEditExport
    },
    {
      label: 'Tags',
      tooltip: 'Tags',
      icon: mdiTagOutline,
      ownerOnly: true,
      pressed: band === 'tags',
      disabled: actions.metaBusy,
      run: () => onBand(band === 'tags' ? null : 'tags')
    },
    {
      label: 'Albums',
      tooltip: 'Albums',
      icon: mdiImageAlbum,
      ownerOnly: false,
      pressed: band === 'albums',
      disabled: actions.metaBusy,
      run: () => onBand(band === 'albums' ? null : 'albums')
    }
  ]);

  function pick(tool: Tool): void {
    menu = false;
    if (tool.ownerOnly && !actions.ownedOrExplain()) return;
    tool.run();
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

<div class="{ROOMY_ONLY} shrink-0 items-center gap-1">
  {#each tools as tool (tool.label)}
    {#if tool.ownerOnly}
      <DisabledReason locked={shared} reason={scope.notOwnedReason} class="flex">
        {@render button(tool)}
      </DisabledReason>
    {:else}
      {@render button(tool)}
    {/if}
  {/each}
</div>
<Popover
  open={menu}
  anchor="top"
  align="end"
  onOpenChange={(open) => (menu = open)}
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
    {#each tools as tool (tool.label)}
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
    {/each}
  </div>
</Popover>
