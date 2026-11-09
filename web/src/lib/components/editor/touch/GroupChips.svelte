<script lang="ts">
  import { runCommand, commandLabel, commandTitle, isEnabled } from '#lib/commands/command.ts';
  import { autoAdjust, resetDevelop } from '#lib/commands/editor/edits.ts';
  import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
  import type { TouchGroupId } from '#lib/editor/params/types.ts';
  import { modifiedTouchGroups } from '#lib/editor/modified.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { Tabs } from 'bits-ui';

  let { grid = false, onretap }: { grid?: boolean; onretap?: () => void } = $props();

  const modified = $derived(modifiedTouchGroups(editor.edits));
</script>

<div class="flex shrink-0 items-center gap-1.5 {grid ? 'px-2 py-2' : 'px-3 py-1.5'}">
  <IconButton
    size="medium"
    variant="ghost"
    color="secondary"
    shape="round"
    icon={autoAdjust.icon}
    title={commandTitle(autoAdjust)}
    aria-label="Auto"
    disabled={!isEnabled(autoAdjust)}
    onclick={() => runCommand(autoAdjust)}
  />
  <IconButton
    size="medium"
    variant="ghost"
    color="secondary"
    shape="round"
    icon={resetDevelop.icon}
    title={commandTitle(resetDevelop)}
    aria-label={commandLabel(resetDevelop)}
    disabled={!isEnabled(resetDevelop)}
    onclick={() => runCommand(resetDevelop)}
  />
  <div class="h-6 w-px shrink-0 bg-hairline"></div>
  <Tabs.Root
    value={touchUi.group}
    onValueChange={(value) => touchUi.setGroup(value as TouchGroupId)}
    activationMode="manual"
    class="min-w-0 flex-1"
  >
    <Tabs.List
      aria-label="Develop groups"
      class={grid ? 'grid grid-cols-6 gap-1' : 'flex gap-1.5 overflow-x-auto scrollbar-hidden'}
    >
      {#each TOUCH_GROUPS as group (group.id)}
        <Tabs.Trigger
          value={group.id}
          onclick={(event) => {
            if (!onretap || group.id !== touchUi.group) return;
            event.stopPropagation();
            onretap();
          }}
          class="relative flex h-9 shrink-0 items-center justify-center gap-1.5 rounded-full px-3.5 text-[13.5px] whitespace-nowrap outline-none focus-visible:ring-2 focus-visible:ring-primary data-[state=active]:bg-primary/22 data-[state=active]:text-light-950 data-[state=inactive]:bg-ghost data-[state=inactive]:text-dark {grid
            ? 'min-w-0 rounded-lg px-1 text-[12px]'
            : ''}"
        >
          {group.label}
          {#if modified.has(group.id)}
            <span
              data-modified-indicator
              aria-hidden="true"
              class="size-1.5 rounded-full bg-primary {grid ? 'absolute top-1.5 right-1.5' : ''}"
            ></span>
          {/if}
        </Tabs.Trigger>
      {/each}
    </Tabs.List>
  </Tabs.Root>
</div>
