<script lang="ts">
  import { chipTagPath, tagPathLabel } from '#lib/browse/tagTree.ts';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import type { TagRef } from '#lib/types/asset.ts';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  let {
    tag,
    disabled = false,
    ruler = false,
    onRemove
  }: {
    tag: TagRef;
    disabled?: boolean;
    ruler?: boolean;
    onRemove: (tagId: string) => void;
  } = $props();

  const path = $derived(chipTagPath(tag.value));
</script>

<span
  class="inline-flex h-6 max-w-full min-w-0 items-center gap-1 rounded-full bg-primary-200 pl-2 text-[11px] whitespace-nowrap touch:h-9 touch:pl-3 touch:text-sm"
>
  {#if tag.color}
    <span class="size-2 shrink-0 rounded-full" style:background-color={tag.color}></span>
  {/if}
  <TruncationTooltip text={tagPathLabel(tag.value)} force={path.shortened}>
    {#snippet child({ props })}
      <span {...props} class="flex min-w-0 items-center gap-1">
        {#if path.parent}
          <span data-truncate class="min-w-0 shrink-[999] truncate text-dark/65">{path.parent}</span
          >
          <span class="shrink-0 text-dark/65">/</span>
        {/if}
        <span data-truncate class="min-w-[3ch] truncate font-medium">{path.leaf}</span>
      </span>
    {/snippet}
  </TruncationTooltip>
  {#if ruler}
    <span class="size-6 shrink-0 touch:size-9"></span>
  {:else}
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      shape="round"
      class="shrink-0 opacity-50 hover:text-red-400 hover:opacity-100 touch:size-9"
      icon={mdiClose}
      title="Remove {tag.value}"
      aria-label="Remove {tag.value}"
      {disabled}
      onclick={() => onRemove(tag.id)}
    />
  {/if}
</span>
