<script lang="ts">
  import MenuItem from '#lib/components/MenuItem.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { ORIGINAL_SOURCE, sameSource, type CompareSource } from '#lib/editor/compare.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiContentDuplicate, mdiEyeOutline, mdiMenuDown, mdiStarOutline } from '@mdi/js';
  import { hasCompareSources, leftName, otherVersions } from './compareText.ts';

  const heading = 'section-label px-3 pt-1.5 pb-1 text-dark/65 touch:px-4';

  let open = $state(false);
  const name = $derived(leftName());
  const versions = $derived(otherVersions());
  const choices = $derived(hasCompareSources());

  function chosen(source: CompareSource): boolean {
    return sameSource(source, editor.compare.source);
  }

  function toggle(next: boolean): void {
    open = next;
    if (next) void editor.versions.load();
  }

  function pick(source: CompareSource): void {
    editor.compare.setSource(source);
    open = false;
  }
</script>

{#snippet current()}
  <span class="truncate">Left: <b class="font-semibold text-dark">{name}</b></span>
{/snippet}

{#if !choices}
  <span class="flex min-w-0 items-center px-2 text-dark/65 touch:text-sm">{@render current()}</span>
{:else}
  <Popover
    {open}
    onOpenChange={toggle}
    anchor="top"
    align="center"
    contentClass="w-60 py-1"
    title="Left side"
    tooltip="Choose the left side"
  >
    {#snippet trigger(props)}
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="min-w-0 gap-1 font-normal text-dark/65 touch:h-10 touch:text-sm"
        title=""
        {...props}
      >
        {@render current()}
        <Icon icon={mdiMenuDown} size="16px" class="shrink-0" aria-hidden="true" />
      </Button>
    {/snippet}
    <div class="flex flex-col">
      <MenuItem
        icon={mdiEyeOutline}
        label="Original"
        pressed={chosen(ORIGINAL_SOURCE)}
        onclick={() => pick(ORIGINAL_SOURCE)}
      />
      {#if editor.snapshots.items.length > 0}
        <span class={heading}>Snapshots</span>
        {#each editor.snapshots.items as snapshot (snapshot.id)}
          {@const source: CompareSource = { kind: 'snapshot', id: snapshot.id }}
          <MenuItem
            icon={mdiStarOutline}
            label={snapshot.name}
            pressed={chosen(source)}
            onclick={() => pick(source)}
          />
        {/each}
      {/if}
      {#if versions.length > 0}
        <span class={heading}>Versions</span>
        {#each versions as version (version.id)}
          {@const source: CompareSource = { kind: 'version', id: version.id }}
          <MenuItem
            icon={mdiContentDuplicate}
            label={version.name}
            pressed={chosen(source)}
            onclick={() => pick(source)}
          />
        {/each}
      {/if}
    </div>
  </Popover>
{/if}
