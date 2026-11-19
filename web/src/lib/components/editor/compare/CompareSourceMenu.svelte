<script lang="ts">
  import MenuItem from '#lib/components/MenuItem.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { ORIGINAL_SOURCE, sameSource, type CompareSource } from '#lib/editor/compare.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiEyeOutline, mdiMenuDown, mdiStarOutline } from '@mdi/js';
  import { leftName } from './compareText.ts';

  const heading = 'px-3 pt-1.5 pb-1 text-[10px] tracking-wider text-dark/65 uppercase touch:px-4';

  let open = $state(false);
  const name = $derived(leftName());

  function chosen(source: CompareSource): boolean {
    return sameSource(source, editor.compare.source);
  }

  function pick(source: CompareSource): void {
    editor.compare.setSource(source);
    open = false;
  }
</script>

<Popover
  {open}
  onOpenChange={(next) => (open = next)}
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
      <span class="truncate">Left: <b class="font-semibold text-dark">{name}</b></span>
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
  </div>
</Popover>
