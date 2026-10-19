import type { RendererChoice, RendererState } from './renderer.svelte';

export interface StatusInput {
  choice: RendererChoice;
  state: RendererState;
  reason: string | null;
  adapter: string | null;
}

export function rendererStatus({ choice, state, reason, adapter }: StatusInput): string {
  if (choice === 'server') return 'Server.';
  if (state === 'browser') {
    const where = `Browser on ${adapter || 'the GPU'}`;
    return choice === 'auto' && reason ? `${where}: ${reason}.` : `${where}.`;
  }
  if (state === 'server' || state === 'measuring') return reason ? `Server: ${reason}.` : 'Server.';
  if (state === 'starting') return 'Starting the browser renderer.';
  if (choice === 'browser') return 'The browser renderer starts when a photo opens.';
  return 'Speed is measured when a photo opens.';
}
