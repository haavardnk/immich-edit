import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
import type { TouchGroup, TouchGroupId } from '#lib/editor/params/types.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

export const TRAY_DETENTS = ['peek', 'compact', 'expanded'] as const;

export type TrayDetent = (typeof TRAY_DETENTS)[number];

export const QUICK_GROUP = 'quick';

export type TrayGroupId = TouchGroupId | typeof QUICK_GROUP;

const STORAGE_KEY = 'immich-edit:touchEditor';
const OPEN_EXPANDED: readonly string[] = ['light/curve', 'color/grading'];

type Persisted = {
  group: string;
  views: Record<string, string>;
  detents: Record<string, string>;
  raised: Record<string, string>;
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

function restoreDetents(stored: unknown): Record<string, TrayDetent> {
  if (!stored || typeof stored !== 'object') return {};
  const detents: Record<string, TrayDetent> = {};
  for (const [key, detent] of Object.entries(stored)) {
    const known = TRAY_DETENTS.find((entry) => entry === detent);
    if (known) detents[key] = known;
  }
  return detents;
}

function defaultDetent(key: string): TrayDetent {
  return OPEN_EXPANDED.includes(key) ? 'expanded' : 'compact';
}

export class TouchUiStore {
  group = $state<TrayGroupId>('light');
  views = $state<Partial<Record<TouchGroupId, string>>>({});
  detents = $state<Record<string, TrayDetent>>({});
  raised = $state<Record<string, TrayDetent>>({});
  focused = $state<string | null>(null);
  searching = $state(false);
  filmstrip = $state(true);

  constructor() {
    const stored = readStored<Persisted>(STORAGE_KEY);
    this.group = restoreGroup(stored?.group);
    this.views = restoreViews(stored?.views);
    this.detents = restoreDetents(stored?.detents);
    this.raised = restoreDetents(stored?.raised);
    this.filmstrip = stored?.filmstrip !== false;
  }

  viewOf = (id: TrayGroupId): string => {
    if (id === QUICK_GROUP) return '';
    return this.views[id] ?? groupOf(id)?.views[0]?.id ?? '';
  };

  get view(): string {
    return this.viewOf(this.group);
  }

  get viewKey(): string {
    return `${this.group}/${this.view}`;
  }

  get detent(): TrayDetent {
    const key = this.viewKey;
    return this.detents[key] ?? defaultDetent(key);
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

  setDetent = (detent: TrayDetent): void => {
    const key = this.viewKey;
    this.detents = { ...this.detents, [key]: detent };
    if (detent !== 'peek') this.raised = { ...this.raised, [key]: detent };
    this.persist();
  };

  raise = (): void => {
    const key = this.viewKey;
    if (this.detent === 'peek') this.setDetent(this.raised[key] ?? defaultDetent(key));
  };

  openSearch = (): void => {
    this.searching = true;
    this.raise();
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
      detents: this.detents,
      raised: this.raised,
      filmstrip: this.filmstrip
    } satisfies Persisted);
  }
}

export const touchUi = new TouchUiStore();
