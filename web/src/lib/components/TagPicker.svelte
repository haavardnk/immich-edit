<script lang="ts">
  import { isManagedTag } from '#lib/browse/managedTags.ts';
  import AddChip from '#lib/components/AddChip.svelte';
  import TagChip from '#lib/components/tags/TagChip.svelte';
  import TagChipStrip from '#lib/components/tags/TagChipStrip.svelte';
  import TagPickerPopover from '#lib/components/tags/TagPickerPopover.svelte';
  import { hint } from '#lib/shortcuts/labels.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiPlus, mdiTagOutline } from '@mdi/js';

  type Anchor = 'top' | 'bottom';
  type Align = 'start' | 'end' | 'center';

  let {
    tags,
    open,
    onToggle,
    onClose,
    onAdd,
    onRemove,
    onCreate,
    disabled = false,
    anchor = 'top',
    align = 'start',
    addLabel,
    subtitle,
    wrap = false
  }: {
    tags: TagRef[];
    open: boolean;
    onToggle: () => void;
    onClose: () => void;
    onAdd: (tag: TagRef) => void | Promise<void>;
    onRemove: (tagId: string) => void | Promise<void>;
    onCreate: (value: string) => TagRef | null | Promise<TagRef | null>;
    disabled?: boolean;
    anchor?: Anchor;
    align?: Align;
    addLabel?: string;
    subtitle?: string;
    wrap?: boolean;
  } = $props();

  const visibleTags = $derived(tags.filter((t) => !isManagedTag(t)));

  function more(): void {
    if (!open) onToggle();
  }
</script>

{#snippet picker(hidden: number)}
  <TagPickerPopover
    {tags}
    {open}
    {onToggle}
    {onClose}
    {onAdd}
    {onRemove}
    {onCreate}
    {disabled}
    {anchor}
    {align}
    {subtitle}
    showApplied={hidden > 0}
    tooltip={addLabel ? undefined : hint('Tags', 'toggleTags')}
  >
    {#snippet trigger(popoverProps)}
      {#if addLabel}
        <AddChip label={addLabel} aria-label="{addLabel} tags" {...popoverProps} {disabled} />
      {:else}
        <IconButton
          size="tiny"
          variant={open ? 'filled' : 'ghost'}
          color={open ? 'primary' : 'secondary'}
          class="shrink-0"
          icon={mdiPlus}
          title=""
          aria-label={hint('Tags', 'toggleTags')}
          {...popoverProps}
          {disabled}
        />
      {/if}
    {/snippet}
  </TagPickerPopover>
{/snippet}

<div class="flex min-w-0 items-center gap-1.5 {wrap ? 'flex-wrap' : 'flex-1 overflow-hidden'}">
  <Icon icon={mdiTagOutline} size="14px" class="opacity-30 shrink-0" aria-hidden="true" />
  {#if wrap}
    <div class="flex min-w-0 flex-wrap items-center gap-1">
      {#each visibleTags as t (t.id)}
        <TagChip tag={t} {disabled} onRemove={(id) => void onRemove(id)} />
      {/each}
    </div>
    {@render picker(0)}
  {:else}
    <TagChipStrip tags={visibleTags} {disabled} onRemove={(id) => void onRemove(id)} onMore={more}>
      {#snippet children(hidden)}
        {@render picker(hidden)}
      {/snippet}
    </TagChipStrip>
  {/if}
</div>
