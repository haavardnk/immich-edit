<script lang="ts">
  import { COLLECTION_ORDER, COLLECTIONS } from '#lib/components/library/collections.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';
</script>

<ul aria-label="Collections" class="flex flex-col">
  {#each COLLECTION_ORDER as id, index (id)}
    {@const collection = COLLECTIONS[id]}
    {@const count = collection.count()}
    <li>
      <a
        href={collection.href}
        class="flex min-h-11 items-center gap-3.5 ps-4 pe-2.5 outline-none hover:bg-subtle focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary sm:ps-5 touch:min-h-14.5"
      >
        <span
          class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-light-200 text-primary touch:size-9.5 touch:rounded-[10px]"
        >
          <Icon icon={collection.icon} size="18px" class="touch:size-5.5" aria-hidden="true" />
        </span>
        <span
          class="flex min-h-11 min-w-0 flex-1 items-center gap-2 self-stretch touch:min-h-14.5 {index <
          COLLECTION_ORDER.length - 1
            ? 'border-b border-hairline'
            : ''}"
        >
          <span class="min-w-0 flex-1 truncate text-sm text-light-950 touch:text-base"
            >{collection.label}</span
          >
          {#if count !== null}
            <span class="shrink-0 text-xs text-dark/65 tabular-nums touch:text-sm">{count}</span>
          {/if}
          <Icon
            icon={mdiChevronRight}
            size="18px"
            class="shrink-0 text-dark/65"
            aria-hidden="true"
          />
        </span>
      </a>
    </li>
  {/each}
</ul>
