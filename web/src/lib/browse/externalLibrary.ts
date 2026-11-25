import { countOf } from '#lib/utils/plural.ts';

export function inExternalLibrary(asset: { libraryId?: string }): boolean {
  return asset.libraryId !== undefined;
}

export function externalLibraryNote(external: number, total: number): string | null {
  if (external === 0) return null;
  const subject = total === 1 ? 'This photo is' : countOf(external, 'photo is', 'photos are');
  const files = external === 1 ? 'its file' : 'their files';
  const it = external === 1 ? 'it' : 'them';
  return `${subject} from an external library. Immich can only delete ${files} with write access to the library folder; without it, the next library scan imports ${it} again.`;
}
