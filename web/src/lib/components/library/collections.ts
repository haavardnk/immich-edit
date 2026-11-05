import { userTags } from '#lib/browse/managedTags.ts';
import { library } from '#lib/stores/library.svelte.ts';
import {
  mdiAccountOutline,
  mdiFolderOutline,
  mdiHeartOutline,
  mdiPencilOutline,
  mdiTagMultipleOutline
} from '@mdi/js';

type CollectionId = 'favorites' | 'people' | 'tags' | 'folders' | 'edited';

type Collection = {
  id: CollectionId;
  label: string;
  href: string;
  icon: string;
  count: () => number | null;
};

export const COLLECTIONS: Readonly<Record<CollectionId, Collection>> = {
  favorites: {
    id: 'favorites',
    label: 'Favorites',
    href: '/favorites',
    icon: mdiHeartOutline,
    count: () => library.favoritesCount
  },
  people: {
    id: 'people',
    label: 'People',
    href: '/people',
    icon: mdiAccountOutline,
    count: () => (library.isLoaded('people') ? library.people.length : null)
  },
  tags: {
    id: 'tags',
    label: 'Tags',
    href: '/tags',
    icon: mdiTagMultipleOutline,
    count: () => (library.isLoaded('tags') ? userTags(library.tags).length : null)
  },
  folders: {
    id: 'folders',
    label: 'Folders',
    href: '/folders',
    icon: mdiFolderOutline,
    count: () => library.foldersCount
  },
  edited: {
    id: 'edited',
    label: 'Edited',
    href: '/edited',
    icon: mdiPencilOutline,
    count: () => library.editedCount
  }
};

export const COLLECTION_ORDER: readonly CollectionId[] = [
  'favorites',
  'people',
  'tags',
  'folders',
  'edited'
];
