<script lang="ts">
  import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
  import type { TouchGroupId } from '#lib/editor/params/types.ts';
  import { modifiedTouchGroups } from '#lib/editor/modified.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { Tabs } from 'bits-ui';

  let { grid = false, onretap }: { grid?: boolean; onretap?: () => void } = $props();

  const modified = $derived(modifiedTouchGroups(editor.edits));
</script>

{#snippet chips()}
  <Tabs.Root
    value={touchUi.group}
    onValueChange={(value) => touchUi.setGroup(value as TouchGroupId)}
    activationMode="manual"
    class="min-w-0 flex-1"
  >
    <Tabs.List
      aria-label="Develop groups"
      class={grid
        ? 'grid grid-cols-6 gap-1'
        : 'flex gap-1.5 overflow-x-auto px-4 scrollbar-hidden fade-x'}
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
{/snippet}

<div class="flex shrink-0 items-center {grid ? 'px-2 py-2' : 'py-1.5'}">
  {@render chips()}
</div>
