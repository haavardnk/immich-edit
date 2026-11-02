import { getAppearance, putAppearance } from '#lib/api/appearance.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { sameAppearance } from '#lib/theme/appearance.ts';
import { buildThemeCache, type ThemeCache } from '#lib/theme/cache.ts';
import {
  pageThemeRevision,
  readThemeCache,
  writeSchemeChrome,
  writeTheme
} from '#lib/theme/document.ts';
import { DEFAULT_APPEARANCE, resolveAppearance } from '#lib/theme/registry.ts';
import type { Appearance, Scheme, ThemeMode } from '#lib/theme/types.ts';
import { Theme, themeManager, ThemePreference } from '@immich/ui';

type AppearanceEdit = (appearance: Appearance) => Appearance;

const PREFERENCE: Readonly<Record<ThemeMode, ThemePreference>> = {
  system: ThemePreference.System,
  light: ThemePreference.Light,
  dark: ThemePreference.Dark
};

class AppearanceStore {
  current = $state<Appearance>(readThemeCache()?.appearance ?? DEFAULT_APPEARANCE);
  #cache: ThemeCache | null = null;
  #mode: ThemeMode | null = null;
  #loading: Promise<void> | null = null;
  #queued: AppearanceEdit[] = [];
  #saving: Promise<void> = Promise.resolve();

  get scheme(): Scheme {
    return themeManager.value === Theme.Dark ? 'dark' : 'light';
  }

  apply(): void {
    this.#apply(this.current);
  }

  load(): Promise<void> {
    const loading = this.#fetch().finally(() => {
      if (this.#loading === loading) this.#loading = null;
    });
    this.#loading = loading;
    return loading;
  }

  reset(): void {
    this.#queued = [];
    this.current = DEFAULT_APPEARANCE;
    this.#apply(DEFAULT_APPEARANCE);
  }

  setMode(mode: ThemeMode): void {
    this.#commit((appearance) => ({ ...appearance, mode }));
  }

  toggleMode(): void {
    this.setMode(this.scheme === 'dark' ? 'light' : 'dark');
  }

  setTheme(scheme: Scheme, id: string): void {
    this.#commit((appearance) =>
      scheme === 'light' ? { ...appearance, lightTheme: id } : { ...appearance, darkTheme: id }
    );
  }

  setAccent(family: string, accent: string): void {
    this.#commit((appearance) => ({
      ...appearance,
      accents: { ...appearance.accents, [family]: accent }
    }));
  }

  showScheme(scheme: Scheme): void {
    if (this.#cache) writeSchemeChrome(this.#cache.chrome[scheme]);
  }

  async #fetch(): Promise<void> {
    const remote = await getAppearance().catch(() => null);
    const queued = this.#queued;
    this.#queued = [];
    const next = remote ? queued.reduce((value, edit) => edit(value), remote) : this.current;
    if (!sameAppearance(next, this.current)) {
      this.current = next;
      this.#apply(next);
    }
    if (queued.length > 0) this.#save(next);
  }

  #apply(appearance: Appearance): void {
    this.#cache = buildThemeCache(appearance, resolveAppearance(appearance), pageThemeRevision());
    writeTheme(this.#cache);
    if (appearance.mode !== this.#mode) {
      themeManager.setPreference(PREFERENCE[appearance.mode]);
      this.#mode = appearance.mode;
    }
    this.showScheme(this.scheme);
  }

  #commit(edit: AppearanceEdit): void {
    const next = edit(this.current);
    this.current = next;
    this.#apply(next);
    if (this.#loading) {
      this.#queued.push(edit);
      return;
    }
    this.#save(next);
  }

  #save(appearance: Appearance): void {
    this.#saving = this.#saving
      .then(() => putAppearance(appearance))
      .then(
        () => undefined,
        (error: unknown) => toasts.fail('Could not save appearance', error)
      );
  }
}

export const appearance = new AppearanceStore();
