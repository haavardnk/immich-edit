<script lang="ts">
  import { COLLECTION_ORDER, COLLECTIONS } from '#lib/components/library/collections.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';
</script>

<ul aria-label="Collections" class="flex flex-col">
  {#each COLLECTION_ORDER as id (id)}
    {@const collection = COLLECTIONS[id]}
    {@const count = collection.count()}
    <li>
      <a
        href={collection.href}
        class="flex min-h-11 items-center gap-4 px-4 outline-none hover:bg-subtle focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary sm:px-5 touch:min-h-13"
      >
        <span
          class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/15 text-primary"
        >
          <Icon icon={collection.icon} size="18px" aria-hidden="true" />
        </span>
        <span
          class="flex min-h-11 min-w-0 flex-1 items-center gap-2 border-b border-hairline touch:min-h-13"
        >
          <span class="min-w-0 flex-1 truncate text-sm text-light-950">{collection.label}</span>
          {#if count !== null}
            <span class="shrink-0 text-xs text-dark/65 tabular-nums">{count}</span>
          {/if}
          <Icon
            icon={mdiChevronRight}
            size="18px"
            class="shrink-0 text-dark/40"
            aria-hidden="true"
          />
        </span>
      </a>
    </li>
  {/each}
</ul>
