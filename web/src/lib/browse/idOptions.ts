import { library } from '#lib/stores/library.svelte.ts';
import { isManagedTag, toTagRef } from './managedTags';
import type { IdFilterKey } from './idFilters';

export interface IdOption {
  id: string;
  label: string;
  count?: number | null;
}

const UNKNOWN_NAMES: Record<IdFilterKey, string> = {
  albums: 'Unknown album',
  people: 'Unknown person',
  tags: 'Unknown tag'
};

function options(key: IdFilterKey): IdOption[] {
  if (key === 'albums')
    return library.albums.map((album) => ({
      id: album.id,
      label: album.albumName,
      count: album.assetCount
    }));
  if (key === 'tags')
    return library.tags
      .filter((tag) => !isManagedTag(toTagRef(tag)))
      .map((tag) => ({ id: tag.id, label: tag.value, count: tag.assetCount }));
  return library.people.map((person) => ({
    id: person.id,
    label: person.name,
    count: person.assetCount
  }));
}

export function idOptions(key: IdFilterKey): IdOption[] | null {
  if (!library.isLoaded(key)) return null;
  return options(key).sort((a, b) => a.label.localeCompare(b.label));
}

export function idName(key: IdFilterKey, id: string): string {
  if (!library.isLoaded(key)) return UNKNOWN_NAMES[key];
  return options(key).find((option) => option.id === id)?.label ?? UNKNOWN_NAMES[key];
}
