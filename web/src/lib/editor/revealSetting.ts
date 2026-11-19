import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';
import { tick } from 'svelte';
import { flashElement } from '#lib/utils/flash.ts';

export interface RevealOptions {
  focus?: boolean;
  fallback?: string;
}

export async function revealSetting(id: string, options: RevealOptions = {}): Promise<void> {
  settingSearch.reveal(id);
  await tick();
  settingSearch.target = null;
  const row =
    document.querySelector<HTMLElement>(`[data-setting="${id}"]:not([data-quick] *)`) ??
    (options.fallback ? document.querySelector<HTMLElement>(options.fallback) : null);
  if (!row) return;
  flashElement(row);
  if (!options.focus) return;
  const control =
    row.querySelector<HTMLElement>('input[type="range"]') ??
    row.querySelector<HTMLElement>('button');
  control?.focus({ preventScroll: true });
}
