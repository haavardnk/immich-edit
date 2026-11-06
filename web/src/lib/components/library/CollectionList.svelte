<script lang="ts">
  import { COLLECTION_ORDER, COLLECTIONS } from '#lib/components/library/collections.ts';
  import LibraryRow from '#lib/components/library/LibraryRow.svelte';
</script>

<ul aria-label="Collections" class="flex flex-col">
  {#each COLLECTION_ORDER as id, index (id)}
    {@const collection = COLLECTIONS[id]}
    <li>
      <LibraryRow
        href={collection.href}
        icon={collection.icon}
        label={collection.label}
        count={collection.count()}
        last={index === COLLECTION_ORDER.length - 1}
      />
    </li>
  {/each}
</ul>
