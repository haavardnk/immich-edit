export type SearchMode = 'description' | 'filename';

const IMAGE_EXT = /\.(arw|cr2|cr3|dng|nef|raf|orf|rw2|jpg|jpeg|png|heic|tif|tiff)$/i;
const CAMERA_NAME = /^_?[a-z]{0,4}_?\d{3,}$/i;

export function looksLikeFilename(query: string): boolean {
  const q = query.trim();
  if (!q || /\s/.test(q)) return false;
  return IMAGE_EXT.test(q) || CAMERA_NAME.test(q);
}

export function resolveSearchMode(query: string, explicit: string | null): SearchMode {
  if (explicit === 'filename' || explicit === 'description') return explicit;
  return looksLikeFilename(query) ? 'filename' : 'description';
}
