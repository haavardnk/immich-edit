import { DESKTOP_PANELS, TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
import type { TouchGroup, TouchView } from '#lib/editor/params/types.ts';
import {
  orderSections,
  reorderSections,
  storedSections,
  toggleSection,
  type SectionEntry
} from '#lib/editor/sectionOrder.ts';
import { preferences } from './preferences.svelte';

type Stored = readonly SectionEntry[] | null;

export function touchViewKey(group: string, view: string): string {
  return `${group}.${view}`;
}

class SectionLayout {
  revealed = $state<string | null>(null);
  readonly #ids: readonly string[];
  readonly #read: () => Stored;
  readonly #write: (next: Stored) => Promise<void>;

  constructor(ids: readonly string[], read: () => Stored, write: (next: Stored) => Promise<void>) {
    this.#ids = ids;
    this.#read = read;
    this.#write = write;
  }

  get entries(): SectionEntry[] {
    return orderSections(this.#ids, this.#read());
  }

  get hiddenCount(): number {
    return this.entries.filter((entry) => entry.hidden).length;
  }

  get customised(): boolean {
    return storedSections(this.entries, this.#ids) !== null;
  }

  isHidden = (id: string): boolean => this.entries.some((entry) => entry.id === id && entry.hidden);

  toggle = (id: string): Promise<void> => this.#save(toggleSection(this.entries, id));

  reorder = (ids: readonly string[]): Promise<void> =>
    this.#save(reorderSections(this.entries, ids));

  reset = (): Promise<void> => this.#write(null);

  #save(entries: readonly SectionEntry[]): Promise<void> {
    return this.#write(storedSections(entries, this.#ids));
  }
}

export const desktopSections = new SectionLayout(
  DESKTOP_PANELS.map((panel) => panel.id),
  () => preferences.desktopSections,
  preferences.setDesktopSections
);

export const touchSections = new SectionLayout(
  TOUCH_GROUPS.flatMap((group) => group.views.map((view) => touchViewKey(group.id, view.id))),
  () => preferences.touchSubviews,
  preferences.setTouchSubviews
);

export interface OrderedView {
  view: TouchView;
  key: string;
  hidden: boolean;
}

export function orderedTouchViews(group: TouchGroup): OrderedView[] {
  return touchSections.entries.flatMap((entry) => {
    const view = group.views.find((item) => touchViewKey(group.id, item.id) === entry.id);
    return view ? [{ view, key: entry.id, hidden: entry.hidden }] : [];
  });
}
