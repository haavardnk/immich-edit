import {
  getPreferences,
  patchPreferences,
  type ClickTarget,
  type Preferences
} from '#lib/api/preferences.ts';
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

class PreferencesStore {
  clickTarget = $state<ClickTarget>(DEFAULT_CLICK_TARGET);
  pinnedSettings = $state.raw<readonly string[] | null>(null);
  pinsKnown = $state(false);
  #clickChosen = false;
  #pinsChosen = false;
  #saving: Promise<void> = Promise.resolve();

  get pins(): readonly string[] {
    if (!this.pinsKnown) return [];
    return this.pinnedSettings ?? DEFAULT_PINNED_SETTINGS;
  }

  async load(): Promise<void> {
    const remote = await getPreferences().catch(() => null);
    this.pinsKnown = true;
    if (!remote) return;
    if (!this.#clickChosen) this.clickTarget = remote.clickTarget;
    if (!this.#pinsChosen) this.pinnedSettings = remote.pinnedSettings;
  }

  reset(): void {
    this.#clickChosen = false;
    this.#pinsChosen = false;
    this.clickTarget = DEFAULT_CLICK_TARGET;
    this.pinnedSettings = null;
    this.pinsKnown = false;
  }

  setClickTarget(target: ClickTarget): Promise<void> {
    const previous = this.clickTarget;
    this.#clickChosen = true;
    this.clickTarget = target;
    return this.#save({ clickTarget: target }, () => {
      if (this.clickTarget === target) this.clickTarget = previous;
    });
  }

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
    const previous = this.pinnedSettings;
    this.#pinsChosen = true;
    this.pinsKnown = true;
    this.pinnedSettings = next;
    return this.#save({ pinnedSettings: [...next] }, () => {
      if (this.pinnedSettings === next) this.pinnedSettings = previous;
    });
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
