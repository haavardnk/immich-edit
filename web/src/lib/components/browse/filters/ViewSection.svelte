<script lang="ts">
  import Disclosure from '#lib/components/Disclosure.svelte';
  import ViewOptions from '#lib/components/browse/ViewOptions.svelte';
  import { VIEW_SECTION } from '#lib/browse/filterSections.ts';
  import { viewSummary } from '#lib/browse/viewOptions.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';

  let { sortBasis }: { sortBasis: 'capture' | 'edit' | null } = $props();

  const open = $derived(browseView.filterSections.includes(VIEW_SECTION.id));
  const summary = $derived(
    viewSummary({
      basis: sortBasis,
      field: browseControls.sortField,
      dir: browseControls.sortDir,
      size: browseView.gridSize,
      info: browseView.tileInfo
    })
  );
</script>

<Disclosure
  {open}
  title={VIEW_SECTION.title}
  {summary}
  onOpenChange={(next) => browseView.setFilterSection(VIEW_SECTION.id, next)}
>
  <div class="flex flex-col gap-4 bg-recess px-4 pt-2 pb-4">
    <ViewOptions {sortBasis} />
  </div>
</Disclosure>
