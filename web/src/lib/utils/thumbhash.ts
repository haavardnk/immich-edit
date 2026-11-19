import { thumbHashToDataURL } from 'thumbhash';

export const THUMBHASH_CACHE_SIZE = 2000;

const decoded = new Map<string, string | null>();

function decode(hash: string): string | null {
  try {
    return thumbHashToDataURL(Uint8Array.from(atob(hash), (char) => char.charCodeAt(0)));
  } catch {
    return null;
  }
}

export function thumbhashDataUrl(hash: string): string | null {
  const cached = decoded.get(hash);
  if (cached !== undefined) {
    decoded.delete(hash);
    decoded.set(hash, cached);
    return cached;
  }
  const dataUrl = decode(hash);
  decoded.set(hash, dataUrl);
  if (decoded.size > THUMBHASH_CACHE_SIZE) {
    const oldest = decoded.keys().next().value;
    if (oldest !== undefined) decoded.delete(oldest);
  }
  return dataUrl;
}
