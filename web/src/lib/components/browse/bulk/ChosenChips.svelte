<script lang="ts">
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  let {
    items,
    onRemove
  }: { items: { id: string; label: string }[]; onRemove: (id: string) => void } = $props();
</script>

{#if items.length > 0}
  <div class="mt-1 flex flex-wrap gap-1">
    {#each items as item (item.id)}
      <span
        class="inline-flex h-8 items-center gap-1 rounded-full bg-light-200 pl-3 text-xs text-dark dark:bg-primary-200 dark:text-white"
      >
        {item.label}
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={mdiClose}
          aria-label="Remove {item.label}"
          onclick={() => onRemove(item.id)}
        />
      </span>
    {/each}
  </div>
{/if}
