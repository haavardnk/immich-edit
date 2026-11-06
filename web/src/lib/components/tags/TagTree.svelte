<script lang="ts">
  import type { TagSummary } from '#lib/api/tags.ts';
  import {
    descendantCount,
    mixedPaths,
    tagCheck,
    tagRows,
    type TagNode
  } from '#lib/browse/tagTree.ts';
  import TagCheckRow from '#lib/components/tags/TagCheckRow.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let {
    nodes,
    applied,
    disabled = false,
    onToggle
  }: {
    nodes: TagNode[];
    applied: ReadonlySet<string>;
    disabled?: boolean;
    onToggle: (tag: TagSummary, on: boolean) => void;
  } = $props();

  let manual = $state<Set<string> | null>(null);

  const mixed = $derived(mixedPaths(nodes, applied));
  const expanded = $derived(manual ?? mixed);
  const rows = $derived(tagRows(nodes, expanded));

  function expand(path: string): void {
    const next = new Set(expanded);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    manual = next;
  }

  function toggle(tag: TagSummary, on: boolean): void {
    manual ??= new Set(expanded);
    onToggle(tag, on);
  }
</script>

<ul class="flex flex-col">
  {#each rows as { node, depth } (node.path)}
    {@const open = expanded.has(node.path)}
    <TagCheckRow
      check={tagCheck(node, applied, mixed)}
      disabled={disabled || !node.tag}
      {depth}
      onChange={(on) => node.tag && toggle(node.tag, on)}
    >
      {#snippet lead()}
        {#if node.children.length > 0}
          <IconButton
            size="tiny"
            variant="ghost"
            color="secondary"
            shape="round"
            class="-ms-1 shrink-0 touch:size-9"
            icon={open ? mdiChevronDown : mdiChevronRight}
            title="{open ? 'Collapse' : 'Expand'} {node.name}"
            aria-label="{open ? 'Collapse' : 'Expand'} {node.name}"
            aria-expanded={open}
            onclick={() => expand(node.path)}
          />
        {:else}
          <span class="-ms-1 size-6 shrink-0 touch:size-9" aria-hidden="true"></span>
        {/if}
      {/snippet}
      <TruncationTooltip text={node.name}>
        {#snippet child({ props })}
          <span {...props} class="min-w-0 truncate">{node.name}</span>
        {/snippet}
      </TruncationTooltip>
      {#if !open && node.children.length > 0}
        <span class="shrink-0 text-dark/65 tabular-nums" aria-hidden="true">
          {descendantCount(node)}
        </span>
      {/if}
    </TagCheckRow>
  {/each}
</ul>
