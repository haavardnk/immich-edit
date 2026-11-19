<script lang="ts">
  import SnapshotNameField from './SnapshotNameField.svelte';
  import type { EditSnapshot } from '#lib/api/snapshots.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiPlus } from '@mdi/js';

  let { oncreated }: { oncreated: (snapshot: EditSnapshot, named: boolean) => void } = $props();

  let naming = $state(false);
  let busy = $state(false);

  async function create(name?: string): Promise<void> {
    if (busy) return;
    busy = true;
    try {
      const created = await editor.snapshots.create(name);
      if (!created) return;
      naming = false;
      oncreated(created, name !== undefined);
    } finally {
      busy = false;
    }
  }

  function start(): void {
    if (layout.mode === 'touch') naming = true;
    else void create();
  }
</script>

{#snippet plus()}
  <span
    class="flex shrink-0 items-center justify-center touch:size-14 touch:rounded-lg touch:border touch:border-dashed touch:border-dark/30"
    aria-hidden="true"
  >
    <Icon
      icon={mdiPlus}
      size="14px"
      class="opacity-70 touch:size-6 touch:text-primary touch:opacity-100"
    />
  </span>
{/snippet}

{#if naming}
  <div
    class="flex items-center gap-2.5 px-3 py-1 touch:gap-3.5 touch:border-b touch:border-hairline touch:px-4 touch:py-2"
  >
    {@render plus()}
    <SnapshotNameField
      initial={editor.snapshots.nextName}
      ariaLabel="Snapshot name"
      onsave={create}
      oncancel={() => (naming = false)}
    />
  </div>
{:else}
  <Button
    type="button"
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-8 w-full justify-start gap-2.5 rounded-none px-3 text-left text-xs font-normal touch:h-auto touch:gap-3.5 touch:border-b touch:border-hairline touch:px-4 touch:py-2"
    aria-label="New snapshot"
    data-snapshot-new
    disabled={busy || !editor.initialised}
    onclick={start}
  >
    {@render plus()}
    <span class="flex min-w-0 flex-1 flex-col touch:gap-0.5">
      <span class="truncate touch:text-base touch:font-medium touch:text-primary">New snapshot</span
      >
      <span class="hidden text-[13px] text-dark/65 touch:block">From the current edits</span>
    </span>
    <kbd
      class="shrink-0 rounded border border-dark/15 px-1 font-mono text-[10px] text-dark/65 touch:hidden"
      aria-hidden="true">{keysFor('newSnapshot')}</kbd
    >
  </Button>
{/if}
