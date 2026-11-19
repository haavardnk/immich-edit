<script lang="ts">
  import { isManagedTag, userTags } from '#lib/browse/managedTags.ts';
  import AddChip from '#lib/components/AddChip.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import TagChip from '#lib/components/tags/TagChip.svelte';
  import TagChipStrip from '#lib/components/tags/TagChipStrip.svelte';
  import TagPickerBody from '#lib/components/tags/TagPickerBody.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiPlus, mdiTagOutline } from '@mdi/js';
  import { onMount } from 'svelte';

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

  onMount(() => {
    library.ensureTags().catch((e: unknown) => toasts.fail('tags', e));
  });

  const sheet = $derived(layout.mode === 'touch');
  const allTags = $derived(userTags(library.tags ?? []));
  const visibleTags = $derived(tags.filter((t) => !isManagedTag(t)));

  function more(): void {
    if (!open) onToggle();
  }
</script>

{#snippet picker(hidden: number)}
  <Popover
    open={open && !disabled}
    {anchor}
    {align}
    onOpenChange={(v) => (v ? onToggle() : onClose())}
    appearance="panel"
    contentClass="flex w-72 flex-col max-h-[min(28rem,var(--bits-popover-content-available-height))]!"
    sheetClass="flex flex-col"
    sheetPicker
    title="Tags"
    {subtitle}
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
    <TagPickerBody
      all={allTags}
      applied={visibleTags}
      showApplied={sheet || hidden > 0}
      numbered={!sheet || layout.hasKeyboard}
      {disabled}
      autofocus={!sheet}
      {onAdd}
      {onRemove}
      {onCreate}
    />
  </Popover>
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
