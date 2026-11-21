import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing, type Removal } from '#lib/stores/browsing.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { selection } from '#lib/stores/selection.svelte.ts';

function neighbourOf(id: string, gone: ReadonlySet<string>): string | null {
  let next = browsing.nextOf(id);
  while (next && gone.has(next.id)) next = browsing.nextOf(next.id);
  if (next) return next.id;
  let prev = browsing.prevOf(id);
  while (prev && gone.has(prev.id)) prev = browsing.prevOf(prev.id);
  return prev?.id ?? null;
}

export function dropFromView(ids: ReadonlySet<string>): Removal {
  const loupe = browseView.loupeId;
  if (loupe && ids.has(loupe)) {
    const next = neighbourOf(loupe, ids);
    if (next) browseView.openLoupe(next);
    else browseView.closeLoupe();
  }
  const active = browseView.activeId;
  if (active && ids.has(active)) browseView.setActive(neighbourOf(active, ids));
  selection.forget(ids);
  libraryChanged();
  return browsing.removeMany(ids);
}
