<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { ui } from '$lib/stores/ui.svelte';
  import { library } from '$lib/stores/library.svelte';
  import AlbumList from '$lib/components/library/AlbumList.svelte';
  import PeopleList from '$lib/components/library/PeopleList.svelte';
  import TagList from '$lib/components/library/TagList.svelte';
  import FolderTree from '$lib/components/library/FolderTree.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import SidebarLink from './SidebarLink.svelte';
  import SidebarSection from './SidebarSection.svelte';
  import {
    mdiImageMultipleOutline,
    mdiImageAlbum,
    mdiFolderOutline,
    mdiAccountOutline,
    mdiHeartOutline,
    mdiTagMultipleOutline,
    mdiPencilOutline,
    mdiChevronRight,
    mdiChevronLeft
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
  class="bg-immich-dark-gray border-r border-white/5 flex flex-col min-h-0 transition-[width] duration-200 ease-out overflow-hidden"
  class:w-64={!ui.leftCollapsed}
  class:w-7={ui.leftCollapsed}
>
  {#if ui.leftCollapsed}
    <button
      class="flex-1 flex items-center justify-center hover:bg-white/5 transition-colors"
      onclick={ui.toggleLeft}
      aria-label="expand library panel"
      title="Library"
    >
      <Icon path={mdiChevronRight} size={16} class="opacity-40" />
    </button>
  {:else}
    <div class="flex items-center border-b border-white/10">
      <span
        class="flex-1 px-4 py-2 text-[11px] uppercase tracking-wider text-immich-dark-fg/40 font-semibold"
        >Library</span
      >
      <button
        class="p-1.5 hover:bg-white/10 transition-colors"
        onclick={ui.toggleLeft}
        aria-label="collapse library panel"
        title="Collapse"
      >
        <Icon path={mdiChevronLeft} size={14} class="opacity-40" />
      </button>
    </div>
    <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
      <SidebarLink
        href="/photos"
        icon={mdiImageMultipleOutline}
        label="Photos"
        count={library.photosCount}
        active={currentPath === '/photos'}
      />
      <div class="border-t border-white/5">
        <SidebarSection
          icon={mdiAccountOutline}
          label="People"
          count={library.people.length}
          expanded={expanded.has('people')}
          onToggle={() => toggleSection('people')}
        >
          <PeopleList />
        </SidebarSection>
      </div>
      <div class="border-t border-white/5">
        <SidebarLink
          href="/favorites"
          icon={mdiHeartOutline}
          label="Favorites"
          count={library.favoritesCount}
          active={currentPath === '/favorites'}
        />
      </div>
      <div class="border-t border-white/5">
        <SidebarSection
          icon={mdiImageAlbum}
          label="Albums"
          count={library.albums.length}
          expanded={expanded.has('albums')}
          onToggle={() => toggleSection('albums')}
        >
          <AlbumList />
        </SidebarSection>
      </div>
      <div class="border-t border-white/5">
        <SidebarSection
          icon={mdiTagMultipleOutline}
          label="Tags"
          count={library.tags.length}
          expanded={expanded.has('tags')}
          onToggle={() => toggleSection('tags')}
        >
          <TagList />
        </SidebarSection>
      </div>
      <div class="border-t border-white/5">
        <SidebarSection
          icon={mdiFolderOutline}
          label="Folders"
          count={library.foldersCount}
          expanded={expanded.has('folders')}
          onToggle={() => toggleSection('folders')}
        >
          <FolderTree nodes={library.folderTree} />
        </SidebarSection>
      </div>
      <div class="border-t border-white/5">
        <SidebarLink
          href="/edited"
          icon={mdiPencilOutline}
          label="Edited"
          count={library.editedCount}
          active={currentPath === '/edited'}
          hideZero
        />
      </div>
    </div>
  {/if}
</aside>
