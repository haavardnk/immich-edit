import { getMapConfig, type MapConfig } from '#lib/api/map.ts';
import { appearance } from '#lib/stores/appearance.svelte.ts';

class MapConfigStore {
  config = $state<MapConfig | null>(null);
  #loading: Promise<void> | null = null;

  readonly style = $derived.by(() => {
    const config = this.config;
    if (!config) return null;
    return appearance.scheme === 'dark' ? config.dark : config.light;
  });

  readonly search = $derived(this.config?.search ?? false);

  ensure(): Promise<void> {
    this.#loading ??= getMapConfig()
      .then((config) => {
        this.config = config;
      })
      .catch((error: unknown) => {
        this.#loading = null;
        throw error;
      });
    return this.#loading;
  }

  reset(): void {
    this.config = null;
    this.#loading = null;
  }
}

export const mapConfig = new MapConfigStore();
