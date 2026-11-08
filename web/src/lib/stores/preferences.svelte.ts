import { getPreferences, patchPreferences, type ClickTarget } from '#lib/api/preferences.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

const DEFAULT_CLICK_TARGET: ClickTarget = 'editor';

class PreferencesStore {
  clickTarget = $state<ClickTarget>(DEFAULT_CLICK_TARGET);
  #chosen = false;
  #saving: Promise<void> = Promise.resolve();

  async load(): Promise<void> {
    const remote = await getPreferences().catch(() => null);
    if (remote && !this.#chosen) this.clickTarget = remote.clickTarget;
  }

  reset(): void {
    this.#chosen = false;
    this.clickTarget = DEFAULT_CLICK_TARGET;
  }

  setClickTarget(target: ClickTarget): Promise<void> {
    const previous = this.clickTarget;
    this.#chosen = true;
    this.clickTarget = target;
    this.#saving = this.#saving
      .then(() => patchPreferences({ clickTarget: target }))
      .then(
        () => undefined,
        (error: unknown) => {
          if (this.clickTarget === target) this.clickTarget = previous;
          toasts.fail('Could not save preferences', error);
        }
      );
    return this.#saving;
  }
}

export const preferences = new PreferencesStore();
