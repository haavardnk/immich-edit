<script lang="ts">
  import { recentTagChord } from '#lib/browse/recentTags.ts';
  import { splitTagValue } from '#lib/browse/tagTree.ts';
  import SectionHeading from '#lib/components/SectionHeading.svelte';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiCheck, mdiPlus } from '@mdi/js';

  let {
    tags,
    applied,
    numbered,
    disabled = false,
    onToggle
  }: {
    tags: TagRef[];
    applied: ReadonlySet<string>;
    numbered: boolean;
    disabled?: boolean;
    onToggle: (tag: TagRef, on: boolean) => void;
  } = $props();

  const id = $props.id();

  function tooltip(tag: TagRef, index: number): string {
    const chord = numbered ? recentTagChord(index) : null;
    return chord ? `${tag.value} (${chord})` : tag.value;
  }
</script>

{#snippet keys()}
  <span class="text-[11px] text-dark/65">{keysFor('recentTag')}</span>
{/snippet}

<section aria-labelledby={id}>
  <SectionHeading {id} title="Recent" level={3} action={numbered ? keys : undefined} />
  <div class="flex flex-wrap gap-1.5 px-4 pb-1 sm:px-5">
    {#each tags as tag, index (tag.id)}
      {@const on = applied.has(tag.id)}
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        shape="round"
        class="h-6 max-w-full gap-1.5 px-2 text-[11px] font-normal touch:h-9 touch:px-3 touch:text-sm {on
          ? 'bg-primary/15 hover:bg-primary/25'
          : 'bg-light-200'}"
        aria-label={tag.value}
        aria-pressed={on}
        title={tooltip(tag, index)}
        {disabled}
        onclick={() => onToggle(tag, !on)}
      >
        {#if numbered}
          <span
            class="grid size-4 shrink-0 place-items-center rounded-sm text-[10px] tabular-nums {on
              ? 'bg-primary text-light'
              : 'bg-current/15'}"
            aria-hidden="true"
          >
            {index + 1}
          </span>
        {:else}
          <Icon
            icon={on ? mdiCheck : mdiPlus}
            size="14px"
            class={on ? 'text-primary' : ''}
            aria-hidden="true"
          />
        {/if}
        <span class="min-w-0 truncate">{splitTagValue(tag.value).leaf}</span>
      </Button>
    {/each}
  </div>
</section>
