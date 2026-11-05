import { browseView } from '#lib/stores/browseView.svelte.ts';
import { tick } from 'svelte';

export async function openFilterPanel(): Promise<void> {
  browseView.setFiltersOpen(true);
  await tick();
  document.getElementById('browse-filters-close')?.focus();
}
