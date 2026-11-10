<script lang="ts">
  import { modifiedTouchGroups } from '#lib/editor/modified.ts';
  import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
  import { paramsModified } from '#lib/editor/params/define.ts';
  import { pinnedParams } from '#lib/editor/quickSettings.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { QUICK_GROUP, touchUi, type TrayGroupId } from '#lib/stores/touchUi.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiPinOutline } from '@mdi/js';
  import { Tabs } from 'bits-ui';

  let { grid = false, onretap }: { grid?: boolean; onretap?: () => void } = $props();

  const modified = $derived(modifiedTouchGroups(editor.edits));
  const quickModified = $derived(
    paramsModified(editor.edits, pinnedParams(preferences.pins), editor.isRaw)
  );
  const chips = $derived<{ id: TrayGroupId; label: string; modified: boolean }[]>([
    { id: QUICK_GROUP, label: 'Quick', modified: quickModified },
    ...TOUCH_GROUPS.map((group) => ({
      id: group.id,
      label: group.label,
      modified: modified.has(group.id)
    }))
  ]);
</script>

<div class="flex shrink-0 items-center {grid ? 'px-2 py-2' : 'py-1.5'}">
  <Tabs.Root
    value={touchUi.group}
    onValueChange={(value) => touchUi.setGroup(value as TrayGroupId)}
    activationMode="manual"
    class="min-w-0 flex-1"
  >
    <Tabs.List
      aria-label="Develop groups"
      class={grid
        ? 'grid grid-cols-7 gap-1'
        : 'flex gap-1.5 overflow-x-auto px-4 scrollbar-hidden fade-x'}
    >
      {#each chips as chip (chip.id)}
        <Tabs.Trigger
          value={chip.id}
          onclick={(event) => {
            if (!onretap || chip.id !== touchUi.group) return;
            event.stopPropagation();
            onretap();
          }}
          class="relative flex h-9 shrink-0 items-center justify-center gap-1.5 rounded-full px-3.5 text-[13.5px] whitespace-nowrap outline-none focus-visible:ring-2 focus-visible:ring-primary data-[state=active]:bg-primary/22 data-[state=active]:text-light-950 data-[state=inactive]:bg-ghost data-[state=inactive]:text-dark {grid
            ? 'min-w-0 rounded-lg px-1 text-[12px]'
            : ''}"
        >
          {#if chip.id === QUICK_GROUP && !grid}
            <Icon icon={mdiPinOutline} size="15px" class="-me-0.5" aria-hidden="true" />
          {/if}
          {chip.label}
          {#if chip.modified}
            <span
              data-modified-indicator
              aria-hidden="true"
              class="size-1.5 rounded-full bg-primary {grid ? 'absolute top-1 right-1' : ''}"
            ></span>
          {/if}
        </Tabs.Trigger>
      {/each}
    </Tabs.List>
  </Tabs.Root>
</div>
