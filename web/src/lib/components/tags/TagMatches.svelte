<script lang="ts">
  import type { TagSummary } from '#lib/api/tags.ts';
  import { splitTagValue, tagPathLabel } from '#lib/browse/tagTree.ts';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TagCheckRow from '#lib/components/tags/TagCheckRow.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Button } from '@immich/ui';
  import { mdiPlus } from '@mdi/js';

  let {
    matches,
    applied,
    query,
    canCreate,
    disabled = false,
    onToggle,
    onCreate
  }: {
    matches: TagSummary[];
    applied: ReadonlySet<string>;
    query: string;
    canCreate: boolean;
    disabled?: boolean;
    onToggle: (tag: TagSummary, on: boolean) => void;
    onCreate: () => void;
  } = $props();
</script>

<ul class="flex flex-col">
  {#each matches as tag (tag.id)}
    {@const path = splitTagValue(tag.value)}
    <TagCheckRow
      check={applied.has(tag.id) ? 'checked' : 'unchecked'}
      {disabled}
      onChange={(on) => onToggle(tag, on)}
    >
      <TruncationTooltip text={tagPathLabel(tag.value)}>
        {#snippet child({ props })}
          <span {...props} data-truncate class="min-w-0 truncate">
            {#if path.parent}<span class="text-dark/65"
                ><HighlightedText text={tagPathLabel(path.parent)} {query} /> /
              </span>{/if}<HighlightedText text={path.leaf} {query} />
          </span>
        {/snippet}
      </TruncationTooltip>
    </TagCheckRow>
  {/each}
</ul>
{#if canCreate}
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-8 w-full justify-start rounded-none px-3 text-left text-xs font-normal touch:h-11 touch:px-4 touch:text-[15px]"
    leadingIcon={mdiPlus}
    {disabled}
    onclick={onCreate}
  >
    <span class="min-w-0 truncate">Create “{query.trim()}”</span>
  </Button>
{:else if matches.length === 0}
  <p class="flex h-8 items-center px-3 text-xs text-muted touch:h-11 touch:px-4">No matches</p>
{/if}
