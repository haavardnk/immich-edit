<script lang="ts">
  import { onMount } from 'svelte';
  import CollectionList from '#lib/components/library/CollectionList.svelte';
  import LibrarySection from '#lib/components/library/LibrarySection.svelte';
  import PeopleStrip from '#lib/components/library/PeopleStrip.svelte';
  import RecentEdits from '#lib/components/library/RecentEdits.svelte';
  import SavedViewList from '#lib/components/savedViews/SavedViewList.svelte';
  import PageFrame from '#lib/components/shell/PageFrame.svelte';
  import { library } from '#lib/stores/library.svelte.ts';
  import { savedViews } from '#lib/stores/savedViews.svelte.ts';

  onMount(() => {
    void library.loadCounts();
    void savedViews.ensure();
  });
</script>

<PageFrame title="Library">
  <LibrarySection
    title="People"
    more={{ href: '/people', label: 'See all', name: 'See all people' }}
  >
    <PeopleStrip />
  </LibrarySection>
  <LibrarySection title="Collections">
    <CollectionList />
  </LibrarySection>
  {#if savedViews.views.length > 0}
    <LibrarySection title="Saved views">
      <SavedViewList variant="list" />
    </LibrarySection>
  {/if}
  <LibrarySection
    title="Recently edited"
    more={{ href: '/edited', label: 'Edited', name: 'All edited photos' }}
  >
    <RecentEdits returnPath="/library" />
  </LibrarySection>
</PageFrame>
