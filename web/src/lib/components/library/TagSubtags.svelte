<script lang="ts">
  import type { TagNode } from '#lib/browse/tagTree.ts';
  import { cssTagColor } from '#lib/browse/tagColor.ts';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import { Icon } from '@immich/ui';
  import { mdiTagOutline } from '@mdi/js';

  let { nodes }: { nodes: readonly TagNode[] } = $props();

  const children = $derived(nodes.filter((node) => node.tag !== null));
</script>

{#if children.length > 0}
  <nav
    aria-label="Sub-tags"
    class="flex items-center gap-2 overflow-x-auto scrollbar-hidden px-3 py-2 sm:px-5"
  >
    <span class="section-label shrink-0 text-dark/65" aria-hidden="true">Sub-tags</span>
    {#each children as node (node.path)}
      {#if node.tag}
        {@const color = cssTagColor(node.tag.color)}
        <TooltipLink
          tooltip={node.name}
          href={`/tags/${node.tag.id}`}
          class="inline-flex h-6 max-w-48 shrink-0 items-center gap-1.5 rounded-full border border-hairline px-2.5 text-caption outline-none hover:bg-subtle hover:text-primary focus-visible:ring-2 focus-visible:ring-primary touch:h-9 touch:px-3 touch:text-sm"
        >
          <span class="flex flex-none {color ? '' : 'opacity-40'}" style:color>
            <Icon icon={mdiTagOutline} size="14px" aria-hidden="true" />
          </span>
          <span data-truncate class="min-w-0 truncate font-medium">{node.name}</span>
          {#if node.tag.assetCount != null}
            <span class="text-dark/65 tabular-nums">{node.tag.assetCount}</span>
          {/if}
        </TooltipLink>
      {/if}
    {/each}
  </nav>
{/if}
