<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { library } from '#lib/stores/library.svelte.ts';
  import { savedViews } from '#lib/stores/savedViews.svelte.ts';
  import { sectionForPath, sidebar, type SidebarSectionId } from '#lib/stores/sidebar.svelte.ts';
  import AlbumList from '#lib/components/library/AlbumList.svelte';
  import PeopleList from '#lib/components/library/PeopleList.svelte';
  import TagSidebarTree from '#lib/components/library/TagSidebarTree.svelte';
  import FolderTree from '#lib/components/library/FolderTree.svelte';
  import SavedViewList from '#lib/components/savedViews/SavedViewList.svelte';
  import SidebarLink from './SidebarLink.svelte';
  import SidebarSection from './SidebarSection.svelte';
  import SidebarHeading from '#lib/components/shell/SidebarHeading.svelte';
  import { COLLECTIONS } from '#lib/components/library/collections.ts';
  import { mdiImageMultipleOutline, mdiImageAlbum } from '@mdi/js';

  const { favorites, people, tags, folders, edited } = COLLECTIONS;

  function toggleSection(id: SidebarSectionId): void {
    if (sidebar.toggle(id)) void library.loadView(id);
  }

  const currentPath = $derived(page.url.pathname);
  const within = $derived(sectionForPath(currentPath));

  onMount(() => {
    void library.load();
    void library.loadCounts();
    void savedViews.ensure();
  });
</script>

<aside
  aria-label="Library"
  class="flex h-full w-64 min-h-0 flex-col overflow-hidden bg-light-50 text-dark wide:w-full"
>
  <SidebarHeading class="shrink-0 wide:hidden">Library</SidebarHeading>
  <div class="flex-1 min-h-0 space-y-1 overflow-y-auto scrollbar-hidden pe-5 wide:px-2.5">
    <SidebarLink
      href="/photos"
      icon={mdiImageMultipleOutline}
      label="Photos"
      count={library.photosCount}
      active={currentPath === '/photos'}
    />
    <SidebarSection
      href={people.href}
      active={currentPath === people.href}
      icon={people.icon}
      label={people.label}
      count={people.count()}
      within={within === 'people'}
      expanded={sidebar.isOpen('people')}
      onToggle={() => toggleSection('people')}
    >
      <PeopleList />
    </SidebarSection>
    <div class="mx-4 my-3 h-px bg-hairline"></div>
    <SidebarLink
      href={favorites.href}
      icon={favorites.icon}
      label={favorites.label}
      count={favorites.count()}
      active={currentPath === favorites.href}
    />
    <SidebarSection
      href="/albums"
      active={currentPath === '/albums'}
      icon={mdiImageAlbum}
      label="Albums"
      count={library.albums.length}
      within={within === 'albums'}
      expanded={sidebar.isOpen('albums')}
      onToggle={() => toggleSection('albums')}
    >
      <AlbumList />
    </SidebarSection>
    <SidebarSection
      href={tags.href}
      active={currentPath === tags.href}
      icon={tags.icon}
      label={tags.label}
      count={tags.count()}
      within={within === 'tags'}
      expanded={sidebar.isOpen('tags')}
      onToggle={() => toggleSection('tags')}
    >
      <TagSidebarTree />
    </SidebarSection>
    <SidebarSection
      href={folders.href}
      active={currentPath === folders.href}
      icon={folders.icon}
      label={folders.label}
      count={folders.count()}
      within={within === 'folders'}
      expanded={sidebar.isOpen('folders')}
      onToggle={() => toggleSection('folders')}
    >
      <FolderTree nodes={library.folderTree} />
    </SidebarSection>
    <SidebarLink
      href={edited.href}
      icon={edited.icon}
      label={edited.label}
      count={edited.count()}
      active={currentPath === edited.href}
      hideZero
    />
    {#if savedViews.views.length > 0}
      <SidebarHeading class="wide:px-3">Saved</SidebarHeading>
      <SavedViewList variant="sidebar" />
    {/if}
  </div>
</aside>
