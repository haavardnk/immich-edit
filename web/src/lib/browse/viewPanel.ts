import { browseView } from '#lib/stores/browseView.svelte.ts';
import { tick } from 'svelte';

export async function openViewPanel(): Promise<void> {
  browseView.setViewOpen(true);
  await tick();
  document.getElementById('browse-view-close')?.focus();
}
