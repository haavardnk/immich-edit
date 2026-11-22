import type { MapMarker } from '#lib/api/map.ts';
import { thumbUrl } from '#lib/api/assets.ts';
import { countOf } from '#lib/utils/plural.ts';

export type Marked = 'none' | 'some' | 'all';

const CHECK_PATH = 'M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z';
const SVG = 'http://www.w3.org/2000/svg';

function clusterSize(count: number): string {
  if (count >= 100) return 'size-13 text-label';
  if (count >= 10) return 'size-10.5 text-caption';
  return 'size-8 text-caption';
}

function badge(count: number): HTMLSpanElement {
  const element = document.createElement('span');
  element.className =
    'absolute -top-2 -right-2 grid h-5 min-w-5 place-items-center rounded-full bg-primary px-1 text-micro font-semibold text-light tabular-nums';
  element.textContent = String(count);
  return element;
}

function check(): SVGSVGElement {
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  svg.setAttribute(
    'class',
    'pointer-events-none absolute -bottom-1.5 -left-1.5 size-5 rounded-full bg-primary fill-light p-0.5 shadow ring-2 ring-white'
  );
  svg.dataset['markCheck'] = '';
  const path = document.createElementNS(SVG, 'path');
  path.setAttribute('d', CHECK_PATH);
  svg.append(path);
  return svg;
}

export function clusterElement(count: number): HTMLButtonElement {
  const element = document.createElement('button');
  element.type = 'button';
  element.dataset['mapCluster'] = String(count);
  element.setAttribute('aria-label', countOf(count, 'photo'));
  element.className = `relative grid place-items-center rounded-full bg-primary font-semibold tabular-nums text-light shadow-lg ring-3 ring-white/60 outline-none focus-visible:ring-primary data-[marked=all]:ring-white data-[marked=some]:ring-white/90 ${clusterSize(count)}`;
  element.textContent = String(count);
  return element;
}

export function photoElement(marker: MapMarker | undefined, count: number): HTMLButtonElement {
  const element = document.createElement('button');
  element.type = 'button';
  element.dataset['mapPhoto'] = marker?.id ?? '';
  element.setAttribute('aria-label', count > 1 ? countOf(count, 'photo') : 'Photo');
  element.className =
    'relative size-12 rounded-lg bg-subtle shadow-lg ring-2 ring-white outline-none focus-visible:ring-primary data-[marked=all]:ring-3 data-[marked=all]:ring-primary data-[marked=some]:ring-primary/60';
  if (marker) {
    const image = document.createElement('img');
    image.src = thumbUrl(marker.id, 'thumbnail');
    image.alt = '';
    image.loading = 'lazy';
    image.draggable = false;
    image.className = 'size-full rounded-lg object-cover';
    element.append(image);
  }
  if (count > 1) element.append(badge(count));
  return element;
}

export function markedOf(ids: readonly string[], selected: (id: string) => boolean): Marked {
  let hits = 0;
  for (const id of ids) if (selected(id)) hits += 1;
  if (hits === 0) return 'none';
  return hits === ids.length ? 'all' : 'some';
}

export function showMarked(element: HTMLElement, marked: Marked): void {
  if ((element.dataset['marked'] ?? 'none') === marked) return;
  element.dataset['marked'] = marked;
  const current = element.querySelector('[data-mark-check]');
  if (marked === 'all' && !current) element.append(check());
  if (marked !== 'all') current?.remove();
}
