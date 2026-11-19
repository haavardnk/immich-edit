import { writeFavicon } from './document';
import { faviconDataUrl, type FaviconBadge } from './favicon';

let accent: string | null = null;
let badge: FaviconBadge | undefined;
let shown = '';

function show(): void {
  if (accent === null) return;
  const href = faviconDataUrl(accent, badge);
  if (href === shown) return;
  shown = href;
  writeFavicon(href);
}

export function showFaviconAccent(next: string): void {
  accent = next;
  show();
}

export function showFaviconBadge(next: FaviconBadge | null): void {
  badge = next ?? undefined;
  show();
}
