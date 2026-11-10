<script lang="ts">
  import DeleteConfirmation from '#lib/components/DeleteConfirmation.svelte';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import SnapshotActions from './SnapshotActions.svelte';
  import NameField from '#lib/components/NameField.svelte';
  import type { SnapshotAction } from './snapshotAction.ts';
  import type { EditSnapshot } from '#lib/api/snapshots.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { clickGuard } from '#lib/gestures/clickGuard.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { longPressRecognizer } from '#lib/gestures/longPress.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { formatAgo, formatDay } from '#lib/utils/datetime.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import {
    mdiCameraRetakeOutline,
    mdiCompare,
    mdiDotsHorizontal,
    mdiPencilOutline,
    mdiStarOutline
  } from '@mdi/js';
  import { tick } from 'svelte';

  let {
    snapshot,
    restored,
    editing,
    onrename,
    onendrename,
    onremoved,
    oncompare
  }: {
    snapshot: EditSnapshot;
    restored: boolean;
    editing: boolean;
    onrename: () => void;
    onendrename: () => void;
    onremoved: () => void;
    oncompare: () => void;
  } = $props();

  let row = $state<HTMLElement | null>(null);
  let restoring = $state(false);
  let deleting = $state(false);
  let actionsOpen = $state(false);

  const store = editor.snapshots;
  const compared = $derived(
    editor.compare.mode !== 'off' &&
      editor.compare.source.kind === 'snapshot' &&
      editor.compare.source.id === snapshot.id
  );
  const touch = $derived(layout.mode === 'touch');
  const thumb = $derived(touch ? store.thumbs.url(snapshot.manifest_hash) : null);
  const sheet = presence(() => actionsOpen);
  const guard = clickGuard();
  const hold = gestures({
    recognizers: [
      longPressRecognizer({
        enabled: () => touch && !editing,
        start: () => {
          guard.arm();
          actionsOpen = true;
        }
      })
    ],
    touchAction: null
  });

  async function focusRow(): Promise<void> {
    await tick();
    row?.querySelector<HTMLElement>('[data-snapshot-restore]')?.focus();
  }

  async function restore(): Promise<void> {
    if (restoring) return;
    restoring = true;
    try {
      await store.restore(snapshot.id);
    } finally {
      restoring = false;
    }
  }

  async function rename(name: string): Promise<void> {
    if (!(await store.rename(snapshot.id, name))) return;
    onendrename();
    await focusRow();
  }

  function cancelRename(): void {
    onendrename();
    void focusRow();
  }

  async function remove(): Promise<void> {
    if (await store.remove(snapshot.id)) onremoved();
  }

  function compare(): void {
    editor.compare.open({ kind: 'snapshot', id: snapshot.id });
    oncompare();
  }

  function act(action: SnapshotAction): void {
    if (action === 'compare') compare();
    else if (action === 'rename') onrename();
    else if (action === 'update') void store.updateWithCurrent(snapshot.id);
    else void remove();
  }
</script>

<li
  bind:this={row}
  class="group flex items-center touch:border-b touch:border-hairline touch:last:border-b-0 {restored
    ? 'bg-primary/10'
    : ''}"
>
  {#if editing}
    <div
      class="flex h-8 min-w-0 flex-1 items-center gap-2.5 pr-1.5 pl-3 touch:h-auto touch:gap-3.5 touch:px-4 touch:py-2"
    >
      <Icon
        icon={mdiStarOutline}
        size="14px"
        class="shrink-0 opacity-70 touch:hidden"
        aria-hidden="true"
      />
      <NameField
        initial={snapshot.name}
        ariaLabel="Snapshot name"
        onsave={rename}
        oncancel={cancelRename}
      />
    </div>
  {:else}
    <div class="flex min-w-0 flex-1" {@attach hold} {@attach guard.attachment}>
      <Button
        type="button"
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-8 min-w-0 flex-1 justify-start gap-2.5 rounded-none pr-1.5 pl-3 text-left text-xs font-normal touch:h-auto touch:gap-3.5 touch:py-2 touch:pr-0 touch:pl-4"
        aria-label="Restore {snapshot.name}"
        data-snapshot-restore
        disabled={restoring}
        onclick={() => void restore()}
        oncontextmenu={(event: MouseEvent) => {
          if (!touch) return;
          event.preventDefault();
          actionsOpen = true;
        }}
      >
        {#if touch}
          <span
            class="size-14 shrink-0 overflow-hidden rounded-lg bg-light-200 {restored
              ? 'ring-2 ring-primary ring-offset-2 ring-offset-light-50'
              : ''}"
            aria-hidden="true"
          >
            {#if thumb}
              <img src={thumb} alt="" class="size-full object-cover" />
            {/if}
          </span>
        {:else}
          <Icon icon={mdiStarOutline} size="14px" class="shrink-0 opacity-70" aria-hidden="true" />
        {/if}
        <span class="flex min-w-0 flex-1 flex-col touch:gap-0.5">
          <span class="flex min-w-0 items-center gap-1.5">
            <span class="truncate text-dark touch:text-base touch:font-medium">{snapshot.name}</span
            >
            {#if restored}
              <span
                class="shrink-0 text-[9px] uppercase tracking-wider text-dark/65 touch:text-[11px] touch:font-semibold touch:text-primary"
                >Restored</span
              >
            {/if}
            {#if compared}
              <span
                class="hidden shrink-0 text-[11px] font-semibold tracking-wider text-primary uppercase touch:inline"
                >Compared</span
              >
            {/if}
          </span>
          <span class="hidden text-[13px] text-dark/65 touch:block">
            {formatDay(snapshot.updated_at)}
          </span>
        </span>
        <span
          class="shrink-0 text-[10px] text-dark/65 touch:hidden {deleting
            ? 'hidden'
            : 'group-hover:hidden group-focus-within:hidden'}"
        >
          {compared ? 'Compared' : formatAgo(snapshot.updated_at)}
        </span>
      </Button>
    </div>
    <div
      class="items-center pr-1.5 touch:hidden {deleting
        ? 'flex'
        : 'hidden group-hover:flex group-focus-within:flex'}"
    >
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiCompare}
        title="Compare with current"
        aria-label="Compare {snapshot.name}"
        disabled={!!editor.geometrySession}
        onclick={compare}
      />
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiPencilOutline}
        title="Rename"
        aria-label="Rename {snapshot.name}"
        onclick={onrename}
      />
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiCameraRetakeOutline}
        title="Update with current"
        aria-label="Update {snapshot.name} with current edits"
        onclick={() => void store.updateWithCurrent(snapshot.id)}
      />
      <DeleteConfirmation
        bind:pending={deleting}
        label="Delete {snapshot.name}"
        title="Delete"
        confirmLabel="Confirm delete {snapshot.name}"
        onconfirm={remove}
      />
    </div>
    <IconButton
      size="medium"
      variant="ghost"
      color="secondary"
      shape="round"
      class="mr-1 hidden touch:inline-flex"
      icon={mdiDotsHorizontal}
      title="Snapshot actions"
      aria-label="Actions for {snapshot.name}"
      onclick={() => (actionsOpen = true)}
    />
  {/if}
  {#if sheet.mounted}
    <SnapshotActions
      open={actionsOpen}
      name={snapshot.name}
      onOpenChange={(next) => (actionsOpen = next)}
      onexited={sheet.exited}
      onaction={act}
    />
  {/if}
</li>
