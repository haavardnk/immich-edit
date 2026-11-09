import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';
import { tick } from 'svelte';
import { prefersReducedMotion } from 'svelte/motion';

export interface RevealOptions {
  focus?: boolean;
  fallback?: string;
}

function flash(row: HTMLElement): void {
  row.removeAttribute('data-flash');
  void row.offsetWidth;
  row.setAttribute('data-flash', '');
  row.addEventListener('animationend', () => row.removeAttribute('data-flash'), { once: true });
}

export async function revealSetting(id: string, options: RevealOptions = {}): Promise<void> {
  settingSearch.reveal(id);
  await tick();
  settingSearch.target = null;
  const row =
    document.querySelector<HTMLElement>(`[data-setting="${id}"]`) ??
    (options.fallback ? document.querySelector<HTMLElement>(options.fallback) : null);
  if (!row) return;
  row.scrollIntoView({
    block: 'center',
    behavior: prefersReducedMotion.current ? 'instant' : 'smooth'
  });
  flash(row);
  if (!options.focus) return;
  const control =
    row.querySelector<HTMLElement>('input[type="range"]') ??
    row.querySelector<HTMLElement>('button');
  control?.focus({ preventScroll: true });
}
