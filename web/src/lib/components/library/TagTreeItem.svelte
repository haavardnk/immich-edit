<script lang="ts">
  import { cssTagColor } from '#lib/browse/tagColor.ts';
  import { descendantCount, type TagTreeRow } from '#lib/browse/tagTree.ts';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import { TAG_TABLE_GRID, type TagDensity } from '#lib/components/library/tagTable.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight, mdiTagOutline } from '@mdi/js';

  let {
    row,
    query,
    density,
    active,
    onToggle
  }: {
    row: TagTreeRow;
    query: string;
    density: TagDensity;
    active: boolean;
    onToggle: () => void;
  } = $props();

  const tag = $derived(row.node.tag);
  const color = $derived(cssTagColor(tag?.color));
  const branch = $derived(row.node.children.length > 0);
  const toggleLabel = $derived(`${row.open ? 'Collapse' : 'Expand'} ${row.node.name}`);
  const table = $derived(density === 'table');
  const onPage = $derived(density !== 'sidebar');
  const subtags = $derived(descendantCount(row.node));

  const ROW: Readonly<Record<TagDensity, string>> = {
    table: `${TAG_TABLE_GRID} h-11 pe-2 touch:h-12`,
    list: 'flex h-12 rounded-lg pe-2',
    sidebar: 'flex rounded-e-full touch:min-h-11'
  };
</script>

<li
  class="items-center {ROW[density]} {active
    ? 'bg-primary/15 text-primary'
    : 'hover:bg-subtle hover:text-primary'}"
>
  <div
    class="flex min-w-0 flex-1 items-center self-stretch"
    style:padding-inline-start="{row.depth * 1.25 + (onPage ? 0.5 : 0.25)}rem"
  >
    {#if tag}
      <TooltipLink
        tooltip={row.node.name}
        href={`/tags/${tag.id}`}
        aria-current={active ? 'page' : undefined}
        class="flex min-w-0 items-center gap-2 self-stretch rounded-md ps-1 pe-2 outline-none {table
          ? 'flex-initial'
          : 'flex-1'} focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary {onPage
          ? 'text-label touch:text-body'
          : 'py-2 text-label'} {row.dimmed ? 'text-dark/45' : ''}"
      >
        <span class="flex flex-none {color ? '' : 'opacity-40'}" style:color>
          <Icon icon={mdiTagOutline} size="16px" aria-hidden="true" />
        </span>
        <span
          data-truncate
          class="min-w-0 truncate leading-tight {table ? '' : 'flex-1'} {table && !row.dimmed
            ? 'font-medium text-light-950'
            : ''}"
        >
          <HighlightedText text={row.node.name} query={row.dimmed ? '' : query} />
        </span>
        {#if !table && tag.assetCount != null}
          <span class="flex-none text-caption text-muted tabular-nums">
            {tag.assetCount}
          </span>
        {/if}
      </TooltipLink>
    {:else}
      <span class="min-w-0 truncate ps-1 pe-2 text-label text-dark/45 {table ? '' : 'flex-1'}">
        {row.node.name}
      </span>
    {/if}
    {#if branch && row.forced}
      <span
        class="grid size-6 shrink-0 place-items-center text-dark/45 touch:size-10"
        aria-hidden="true"
      >
        <Icon icon={mdiChevronDown} size="16px" />
      </span>
    {:else if branch}
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        shape="round"
        class="shrink-0 touch:size-10"
        icon={row.open ? mdiChevronDown : mdiChevronRight}
        title={toggleLabel}
        aria-label={toggleLabel}
        aria-expanded={row.open}
        onclick={onToggle}
      />
    {:else if !table}
      <span class="size-6 shrink-0 touch:size-10" aria-hidden="true"></span>
    {/if}
    {#if table}
      <span class="flex-1" aria-hidden="true"></span>
    {/if}
  </div>
  {#if table}
    <span class="text-end text-caption text-dark/65 tabular-nums">
      {#if subtags > 0}{subtags}<span class="sr-only"> sub-tags</span>{:else}<span
          aria-hidden="true"
        >
          –
        </span>{/if}
    </span>
    <span class="text-end text-caption text-dark/80 tabular-nums">
      {#if tag?.assetCount != null}{tag.assetCount}<span class="sr-only"> photos</span>{/if}
    </span>
  {/if}
  {#if table}
    <span aria-hidden="true"></span>
  {/if}
</li>
