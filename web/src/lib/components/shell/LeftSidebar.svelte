<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { library } from '$lib/stores/library.svelte';
  import AlbumList from '$lib/components/library/AlbumList.svelte';
  import PeopleList from '$lib/components/library/PeopleList.svelte';
  import TagList from '$lib/components/library/TagList.svelte';
  import FolderTree from '$lib/components/library/FolderTree.svelte';
  import SidebarLink from './SidebarLink.svelte';
  import SidebarSection from './SidebarSection.svelte';
  import {
    mdiImageMultipleOutline,
    mdiImageAlbum,
    mdiFolderOutline,
    mdiAccountOutline,
    mdiHeartOutline,
    mdiTagMultipleOutline,
    mdiPencilOutline
  } from '@mdi/js';

  type ExpandableSection = 'people' | 'albums' | 'tags' | 'folders';

  let expanded = $state(new Set<ExpandableSection>());

  function toggleSection(id: ExpandableSection): void {
    if (expanded.has(id)) {
      expanded.delete(id);
    } else {
      expanded.add(id);
      void library.loadView(id);
    }
    expanded = new Set(expanded);
  }

  const currentPath = $derived(page.url.pathname);

  onMount(() => {
    void library.load();
    void library.loadCounts();
  });
</script>

<aside
  aria-label="Library"
  class="flex h-full w-64 min-h-0 flex-col overflow-hidden bg-neutral-950 text-dark"
>
  <div class="shrink-0 px-5 pb-2 pt-5 text-[10px] font-semibold uppercase text-white/35">
    Library
  </div>
  <div class="flex-1 min-h-0 space-y-1 overflow-y-auto scrollbar-hidden pe-5">
    <SidebarLink
      href="/photos"
      icon={mdiImageMultipleOutline}
      label="Photos"
      count={library.photosCount}
      active={currentPath === '/photos'}
    />
    <SidebarSection
      icon={mdiAccountOutline}
      label="People"
      count={library.people.length}
      expanded={expanded.has('people')}
      onToggle={() => toggleSection('people')}
    >
      <PeopleList />
    </SidebarSection>
    <div class="mx-4 my-3 h-px bg-hairline"></div>
    <SidebarLink
      href="/favorites"
      icon={mdiHeartOutline}
      label="Favorites"
      count={library.favoritesCount}
      active={currentPath === '/favorites'}
    />
    <SidebarSection
      icon={mdiImageAlbum}
      label="Albums"
      count={library.albums.length}
      expanded={expanded.has('albums')}
      onToggle={() => toggleSection('albums')}
    >
      <AlbumList />
    </SidebarSection>
    <SidebarSection
      icon={mdiTagMultipleOutline}
      label="Tags"
      count={library.tags.length}
      expanded={expanded.has('tags')}
      onToggle={() => toggleSection('tags')}
    >
      <TagList />
    </SidebarSection>
    <SidebarSection
      icon={mdiFolderOutline}
      label="Folders"
      count={library.foldersCount}
      expanded={expanded.has('folders')}
      onToggle={() => toggleSection('folders')}
    >
      <FolderTree nodes={library.folderTree} />
    </SidebarSection>
    <SidebarLink
      href="/edited"
      icon={mdiPencilOutline}
      label="Edited"
      count={library.editedCount}
      active={currentPath === '/edited'}
      hideZero
    />
  </div>
</aside>
