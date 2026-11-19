<script lang="ts">
  import { coverageOf, favoriteMark, labelLook, rejectMark } from '#lib/browse/bulk/bulkMarks.ts';
  import type { CullActions, CullState } from '#lib/browse/cullState.ts';
  import type { TagActions } from '#lib/browse/loupeActions.ts';
  import { isManagedTag } from '#lib/browse/managedTags.ts';
  import { NOT_OWNER } from '#lib/browse/ownership.ts';
  import BarAction from '#lib/components/browse/BarAction.svelte';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import LabelPicker from '#lib/components/LabelPicker.svelte';
  import StarRating from '#lib/components/StarRating.svelte';
  import TagPickerPopover from '#lib/components/tags/TagPickerPopover.svelte';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { LABEL_NAMES } from '#lib/stores/labels.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import { mdiTagOutline } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    cull,
    actions,
    tags,
    tagActions,
    fileName,
    density,
    trailing
  }: {
    cull: CullState;
    actions: CullActions;
    tags: TagRef[];
    tagActions: TagActions;
    fileName?: string;
    density: 'stacked' | 'row';
    trailing?: Snippet;
  } = $props();

  const stacked = $derived(density === 'stacked');
  const locked = $derived(!cull.owned);
  const favorite = $derived(favoriteMark(coverageOf(cull.isFavorite)));
  const reject = $derived(rejectMark(coverageOf(cull.rejected)));
  const label = $derived(labelLook(cull.label));
  const tagCount = $derived(tags.filter((tag) => !isManagedTag(tag)).length);
  const action = $derived(stacked ? 'max-w-24' : 'h-12 w-16 flex-none');
</script>

<nav
  aria-label="Photo actions"
  data-density={density}
  class="shrink-0 border-t border-hairline bg-editor-chrome {stacked
    ? 'grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-2 px-2 pt-1 pb-(--safe-bottom)'
    : 'flex h-16 items-center gap-3 px-3'}"
>
  <DisabledReason
    {locked}
    reason={NOT_OWNER}
    class="flex shrink-0 {stacked ? 'col-span-2 justify-center py-1' : ''}"
  >
    <StarRating
      rating={cull.rating}
      size={stacked ? 30 : 24}
      disabled={locked}
      onchange={actions.rate}
    />
  </DisabledReason>
  <DisabledReason
    {locked}
    reason={NOT_OWNER}
    class="flex min-w-0 items-center {stacked
      ? 'justify-center'
      : 'gap-1 border-l border-hairline pl-3'}"
  >
    <BarAction
      class={action}
      icon={favorite.icon}
      iconClass={favorite.tone}
      label={favorite.label}
      aria-pressed={favorite.pressed}
      disabled={locked}
      onclick={actions.favorite}
    />
    <BarAction
      class={action}
      icon={reject.icon}
      iconClass={reject.tone}
      label={reject.label}
      aria-pressed={reject.pressed}
      disabled={locked}
      onclick={actions.reject}
    />
    <LabelPicker label={cull.label} disabled={locked} onchange={actions.label}>
      {#snippet trigger(props)}
        <BarAction
          {...props}
          class={action}
          icon={label.icon}
          iconClass={label.tone}
          label="Label"
          aria-label={cull.label ? `${LABEL_NAMES[cull.label]} label` : 'Colour label'}
          data-label={cull.label ?? undefined}
          disabled={locked}
        />
      {/snippet}
    </LabelPicker>
    <TagPickerPopover
      {tags}
      disabled={locked}
      open={browseView.loupeTagsOpen}
      onToggle={() => (browseView.loupeTagsOpen = !browseView.loupeTagsOpen)}
      onClose={() => (browseView.loupeTagsOpen = false)}
      onAdd={tagActions.add}
      onRemove={tagActions.remove}
      onCreate={tagActions.create}
      subtitle={fileName}
    >
      {#snippet trigger(props)}
        <BarAction
          {...props}
          class={action}
          icon={mdiTagOutline}
          label="Tags"
          badge={tagCount}
          disabled={locked}
        />
      {/snippet}
    </TagPickerPopover>
  </DisabledReason>
  {#if trailing}
    <div class="flex items-center gap-1 {stacked ? '' : 'ml-auto'}">
      {@render trailing()}
    </div>
  {/if}
</nav>
