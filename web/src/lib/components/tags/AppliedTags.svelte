<script lang="ts">
  import SectionHeading from '#lib/components/SectionHeading.svelte';
  import TagChip from '#lib/components/tags/TagChip.svelte';
  import type { TagRef } from '#lib/types/asset.ts';
  import { Button } from '@immich/ui';

  let {
    tags,
    disabled = false,
    onRemove
  }: { tags: TagRef[]; disabled?: boolean; onRemove: (tagId: string) => void } = $props();

  const id = $props.id();

  function clearAll(): void {
    for (const tag of tags) onRemove(tag.id);
  }
</script>

{#snippet clear()}
  <Button
    size="tiny"
    variant="ghost"
    color="primary"
    class="-me-2 touch:min-h-11 touch:text-[12.5px]"
    {disabled}
    onclick={clearAll}
  >
    Clear
  </Button>
{/snippet}

<section aria-labelledby={id}>
  <SectionHeading {id} title="Applied" level={3} action={clear} />
  <div class="flex flex-wrap gap-1.5 px-4 pb-1 sm:px-5">
    {#each tags as tag (tag.id)}
      <TagChip {tag} {disabled} {onRemove} />
    {/each}
  </div>
</section>
