<script lang="ts">
  import { recentTagChord } from '#lib/browse/recentTags.ts';
  import { splitTagValue } from '#lib/browse/tagTree.ts';
  import Chip from '#lib/components/chips/Chip.svelte';
  import SectionHeading from '#lib/components/SectionHeading.svelte';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import { Icon } from '@immich/ui';
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
  <span class="text-caption text-dark/65">{keysFor('recentTag')}</span>
{/snippet}

<section aria-labelledby={id}>
  <SectionHeading {id} title="Recent" level={3} action={numbered ? keys : undefined} />
  <div class="flex flex-wrap gap-1.5 px-4 pb-1 sm:px-5">
    {#each tags as tag, index (tag.id)}
      {@const on = applied.has(tag.id)}
      <Chip
        pressed={on}
        onPressedChange={(next) => onToggle(tag, next)}
        label={tag.value}
        title={tooltip(tag, index)}
        {disabled}
        class="max-w-full gap-1.5"
      >
        {#if numbered}
          <span
            class="grid size-4 shrink-0 place-items-center rounded-sm text-micro tabular-nums {on
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
      </Chip>
    {/each}
  </div>
</section>
