import { backParent } from '#lib/components/shell/backParent.ts';
import { isEditorPath } from '#lib/utils/route.ts';

export type PageDirection = 'none' | 'tab' | 'forward' | 'back';
export type PageMotion = 'fade' | 'push' | 'pop' | 'rise' | 'sink';

const MAX_DEPTH = 8;

function depth(url: URL, roots: ReadonlySet<string>): number {
  let current = url;
  let level = 0;
  while (!roots.has(current.pathname) && level < MAX_DEPTH) {
    current = new URL(backParent(current), current);
    level += 1;
  }
  return level;
}

export function pageDirection(from: URL, to: URL, roots: ReadonlySet<string>): PageDirection {
  if (from.pathname === to.pathname) return 'none';
  if (isEditorPath(from.pathname) || isEditorPath(to.pathname)) return 'none';
  const before = depth(from, roots);
  const after = depth(to, roots);
  if (after > before) return 'forward';
  if (after < before) return 'back';
  return 'tab';
}

export function pageMotion(direction: PageDirection, compact: boolean): PageMotion | null {
  if (direction === 'none') return null;
  if (direction === 'tab') return 'fade';
  if (direction === 'forward') return compact ? 'push' : 'rise';
  return compact ? 'pop' : 'sink';
}
