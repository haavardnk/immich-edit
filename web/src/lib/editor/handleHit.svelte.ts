import { layout } from '#lib/device/layout.svelte.ts';
import { HANDLE_HIT } from '#lib/editor/handles.ts';

export function handleHit(): number {
  return HANDLE_HIT[layout.mode];
}
