import type { Command } from '#lib/commands/types.ts';
import { toggleDocumentFullscreen } from '#lib/viewer/fullscreen.ts';

export const VIEWER_COMMANDS: readonly Command[] = [
  {
    id: 'viewer.fullscreen',
    contexts: ['viewer'],
    bind: 'fullscreen',
    run: toggleDocumentFullscreen
  }
];
