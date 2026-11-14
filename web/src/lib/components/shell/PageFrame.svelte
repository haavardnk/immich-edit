<script lang="ts">
  import { page } from '$app/state';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
  import { gridPath } from '#lib/browse/gridReturn.ts';
  import { rememberScroll } from '#lib/navigation/scrollMemory.ts';
  import type { Snippet } from 'svelte';

  let {
    title,
    count,
    actions,
    toolbar,
    children
  }: {
    title: string;
    count?: string;
    actions?: Snippet;
    toolbar?: Snippet;
    children: Snippet;
  } = $props();

  const scrollKey = gridPath(page.url);
</script>

<div class="flex min-h-0 min-w-0 flex-1 flex-col">
  <PageHeader {title} {count} {actions} compactActions={actions} />
  {@render toolbar?.()}
  <div
    class="min-h-0 flex-1 overflow-y-auto scrollbar-hidden"
    data-testid="page-scroller"
    {@attach rememberScroll(scrollKey)}
  >
    <div>
      {@render children()}
    </div>
  </div>
</div>
