import { swapView } from '#lib/motion/viewSwap.ts';
import { ui } from '#lib/stores/ui.svelte.ts';

export function toggleEditorFullscreen(): void {
  swapView('fullscreen', ui.toggleFullscreen);
}
