import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';

export type TextField = 'filename' | 'description' | 'path' | 'ocr';
export type SearchMode = 'context' | TextField;

export const SEARCH_MODES: { mode: SearchMode; label: string }[] = [
  { mode: 'context', label: 'Context' },
  { mode: 'filename', label: 'File name' },
  { mode: 'description', label: 'Description' },
  { mode: 'path', label: 'Full path or folder' },
  { mode: 'ocr', label: 'OCR' }
];

const IMAGE_EXT = /\.(arw|cr2|cr3|dng|nef|raf|orf|rw2|jpg|jpeg|png|heic|tif|tiff)$/i;
const CAMERA_NAME = /^_?[a-z]{0,4}_?\d{3,}$/i;

export function looksLikeFilename(query: string): boolean {
  const q = query.trim();
  if (!q || /\s/.test(q)) return false;
  return IMAGE_EXT.test(q) || CAMERA_NAME.test(q);
}

export function searchModeAvailable(mode: SearchMode): boolean {
  if (mode === 'context') return immichCapabilities.smartSearch;
  if (mode === 'ocr') return immichCapabilities.ocr;
  return true;
}

export function resolveSearchMode(query: string, explicit: string | null): SearchMode {
  const chosen = SEARCH_MODES.find(({ mode }) => mode === explicit)?.mode;
  if (chosen && searchModeAvailable(chosen)) return chosen;
  return looksLikeFilename(query) || !immichCapabilities.smartSearch ? 'filename' : 'context';
}
