<script lang="ts">
  import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
  import { modifiedTouchViews } from '#lib/editor/modified.ts';
  import { touchViewParts } from '#lib/panels/touchViews.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { QUICK_GROUP, touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { Tabs } from 'bits-ui';
  import DevelopTools from './DevelopTools.svelte';
  import QuickView from './QuickView.svelte';

  let { stacked = false }: { stacked?: boolean } = $props();

  const group = $derived(TOUCH_GROUPS.find((entry) => entry.id === touchUi.group));
  const views = $derived(group?.views ?? []);
  const shown = $derived(stacked ? views : views.filter((view) => view.id === touchUi.view));
  const modified = $derived(group ? modifiedTouchViews(editor.edits, group.id) : new Set<string>());

  function anchorOf(view: string): string {
    return `touch-view-${touchUi.group}-${view}`;
  }

  function select(view: string): void {
    touchUi.setView(view);
    if (stacked) document.getElementById(anchorOf(view))?.scrollIntoView({ block: 'start' });
  }
</script>

{#if touchUi.group === QUICK_GROUP}
  <QuickView />
{:else if group}
  <Tabs.Root
    value={touchUi.view}
    onValueChange={select}
    class="flex shrink-0 items-center border-b border-hairline pr-2"
  >
    <Tabs.List
      aria-label="{group.label} views"
      class="flex h-10 min-w-0 flex-1 items-stretch gap-5 overflow-x-auto px-4 scrollbar-hidden"
    >
      {#each views as view (view.id)}
        <Tabs.Trigger
          value={view.id}
          class="flex shrink-0 items-center gap-1.5 border-b-2 text-[13.5px] whitespace-nowrap outline-none focus-visible:text-primary data-[state=active]:border-primary data-[state=active]:font-medium data-[state=active]:text-light-950 data-[state=inactive]:border-transparent data-[state=inactive]:text-dark/65"
        >
          {view.label}
          {#if modified.has(view.id)}
            <span
              data-modified-indicator
              aria-hidden="true"
              class="size-1.5 rounded-full bg-primary"
            ></span>
          {/if}
        </Tabs.Trigger>
      {/each}
    </Tabs.List>
    <DevelopTools size="small" />
  </Tabs.Root>
  <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain scrollbar-hidden">
    {#each shown as view (view.id)}
      <section
        id={anchorOf(view.id)}
        aria-label={view.label}
        class="flex scroll-mt-2 flex-col divide-y divide-dark/10 px-3 py-1.5"
      >
        {#each touchViewParts(group.id, view.id) as part, index (index)}
          <div class="flex flex-col gap-1 py-1.5">
            <part.component {...part.props} />
          </div>
        {/each}
      </section>
    {/each}
  </div>
{/if}
