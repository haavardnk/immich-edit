import {
  getPreferences,
  patchPreferences,
  type ClickTarget,
  type Preferences
} from '#lib/api/preferences.ts';
import type { SectionEntry } from '#lib/editor/sectionOrder.ts';
import type { ShortcutMap } from '#lib/shortcuts/custom.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

const DEFAULT_CLICK_TARGET: ClickTarget = 'editor';
export const DEFAULT_PINNED_SETTINGS: readonly string[] = [
  'exposure',
  'contrast',
  'shadows',
  'vibrance',
  'clarity',
  'sharpen_amount'
];
export const MAX_PINNED_SETTINGS = 64;

type Field = keyof Preferences;

class PreferencesStore implements Preferences {
  clickTarget = $state<ClickTarget>(DEFAULT_CLICK_TARGET);
  pinnedSettings = $state.raw<readonly string[] | null>(null);
  shortcuts = $state.raw<ShortcutMap>({});
  desktopSections = $state.raw<readonly SectionEntry[] | null>(null);
  touchSubviews = $state.raw<readonly SectionEntry[] | null>(null);
  pinsKnown = $state(false);
  #chosen = new Set<Field>();
  #saving: Promise<void> = Promise.resolve();

  get pins(): readonly string[] {
    if (!this.pinsKnown) return [];
    return this.pinnedSettings ?? DEFAULT_PINNED_SETTINGS;
  }

  async load(): Promise<void> {
    const remote = await getPreferences().catch(() => null);
    this.pinsKnown = true;
    if (!remote) return;
    if (!this.#chosen.has('clickTarget')) this.clickTarget = remote.clickTarget;
    if (!this.#chosen.has('pinnedSettings')) this.pinnedSettings = remote.pinnedSettings;
    if (!this.#chosen.has('shortcuts')) this.shortcuts = remote.shortcuts;
    if (!this.#chosen.has('desktopSections')) this.desktopSections = remote.desktopSections;
    if (!this.#chosen.has('touchSubviews')) this.touchSubviews = remote.touchSubviews;
  }

  reset(): void {
    this.#chosen.clear();
    this.clickTarget = DEFAULT_CLICK_TARGET;
    this.pinnedSettings = null;
    this.shortcuts = {};
    this.desktopSections = null;
    this.touchSubviews = null;
    this.pinsKnown = false;
  }

  setClickTarget(target: ClickTarget): Promise<void> {
    return this.#write('clickTarget', target);
  }

  setShortcuts = (next: ShortcutMap): Promise<void> => this.#write('shortcuts', next);

  setDesktopSections = (next: readonly SectionEntry[] | null): Promise<void> =>
    this.#write('desktopSections', next);

  setTouchSubviews = (next: readonly SectionEntry[] | null): Promise<void> =>
    this.#write('touchSubviews', next);

  isPinned = (id: string): boolean => this.pins.includes(id);

  togglePin = (id: string): Promise<void> => {
    const pins = this.pins;
    if (pins.includes(id)) return this.#setPins(pins.filter((pin) => pin !== id));
    if (pins.length >= MAX_PINNED_SETTINGS) {
      toasts.push('info', `Quick holds up to ${MAX_PINNED_SETTINGS} settings`, { ttlMs: 3000 });
      return Promise.resolve();
    }
    return this.#setPins([...pins, id]);
  };

  #setPins(next: readonly string[]): Promise<void> {
    this.pinsKnown = true;
    return this.#write('pinnedSettings', next);
  }

  #write<K extends Field>(field: K, value: Preferences[K]): Promise<void> {
    const previous = this.#read(field);
    this.#chosen.add(field);
    this.#assign(field, value);
    const patch: Partial<Preferences> = {};
    patch[field] = value;
    return this.#save(patch, () => {
      if (this.#read(field) === value) this.#assign(field, previous);
    });
  }

  #read<K extends Field>(field: K): Preferences[K] {
    return (this as Preferences)[field];
  }

  #assign<K extends Field>(field: K, value: Preferences[K]): void {
    (this as Preferences)[field] = value;
  }

  #save(patch: Partial<Preferences>, rollback: () => void): Promise<void> {
    this.#saving = this.#saving
      .then(() => patchPreferences(patch))
      .then(
        () => undefined,
        (error: unknown) => {
          rollback();
          toasts.fail('Could not save preferences', error);
        }
      );
    return this.#saving;
  }
}

export const preferences = new PreferencesStore();
