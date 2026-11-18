<script lang="ts">
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
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
</script>

<div class="flex min-h-0 min-w-0 flex-1 flex-col">
  <PageHeader {title} {count} {actions} compactActions={actions} />
  {@render toolbar?.()}
  <div class="min-h-0 flex-1 overflow-y-auto scrollbar-hidden">
    {@render children()}
  </div>
</div>
