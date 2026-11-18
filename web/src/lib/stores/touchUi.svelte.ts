import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
import type { TouchGroup, TouchGroupId } from '#lib/editor/params/types.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

export const TRAY_DETENTS = ['compact', 'expanded'] as const;

export type TrayDetent = (typeof TRAY_DETENTS)[number];

export const QUICK_GROUP = 'quick';

export type TrayGroupId = TouchGroupId | typeof QUICK_GROUP;

const STORAGE_KEY = 'immich-edit:touchEditor';

type Persisted = {
  group: string;
  views: Record<string, string>;
  detent: string;
  filmstrip: boolean;
};

function groupOf(id: unknown): TouchGroup | undefined {
  return TOUCH_GROUPS.find((group) => group.id === id);
}

function restoreGroup(id: unknown): TrayGroupId {
  return id === QUICK_GROUP ? QUICK_GROUP : (groupOf(id)?.id ?? 'light');
}

function restoreViews(stored: unknown): Partial<Record<TouchGroupId, string>> {
  if (!stored || typeof stored !== 'object') return {};
  const views: Partial<Record<TouchGroupId, string>> = {};
  for (const [id, view] of Object.entries(stored)) {
    const group = groupOf(id);
    if (group?.views.some((entry) => entry.id === view)) views[group.id] = view;
  }
  return views;
}

function restoreDetent(stored: unknown): TrayDetent {
  return TRAY_DETENTS.find((entry) => entry === stored) ?? 'compact';
}

export class TouchUiStore {
  group = $state<TrayGroupId>('light');
  views = $state<Partial<Record<TouchGroupId, string>>>({});
  detent = $state<TrayDetent>('compact');
  focused = $state<string | null>(null);
  searching = $state(false);
  filmstrip = $state(true);

  constructor() {
    const stored = readStored<Persisted>(STORAGE_KEY);
    this.group = restoreGroup(stored?.group);
    this.views = restoreViews(stored?.views);
    this.detent = restoreDetent(stored?.detent);
    this.filmstrip = stored?.filmstrip !== false;
  }

  viewOf = (id: TrayGroupId): string => {
    if (id === QUICK_GROUP) return '';
    return this.views[id] ?? groupOf(id)?.views[0]?.id ?? '';
  };

  get view(): string {
    return this.viewOf(this.group);
  }

  setGroup = (id: TrayGroupId): void => {
    this.group = id;
    this.persist();
  };

  setView = (id: string): void => {
    if (this.group === QUICK_GROUP) return;
    this.views = { ...this.views, [this.group]: id };
    this.persist();
  };

  setGroupView = (group: TouchGroupId, view: string): void => {
    this.views = { ...this.views, [group]: view };
    this.persist();
  };

  setDetent = (detent: TrayDetent): void => {
    this.detent = detent;
    this.persist();
  };

  openSearch = (): void => {
    this.searching = true;
  };

  closeSearch = (): void => {
    this.searching = false;
  };

  focus = (id: string | null): void => {
    this.focused = id;
  };

  toggleFilmstrip = (): void => {
    this.filmstrip = !this.filmstrip;
    this.persist();
  };

  private persist(): void {
    writeStored(STORAGE_KEY, {
      group: this.group,
      views: this.views,
      detent: this.detent,
      filmstrip: this.filmstrip
    } satisfies Persisted);
  }
}

export const touchUi = new TouchUiStore();
