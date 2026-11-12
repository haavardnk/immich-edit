import { greyCanvas, help } from '#lib/commands/app.ts';
import { editorTrash } from '#lib/commands/editor/culling.ts';
import {
  autoAdjust,
  copyEdits,
  newSnapshot,
  pasteEdits,
  pasteOnly,
  redo,
  resetDevelop,
  undo,
  virtualCopy
} from '#lib/commands/editor/edits.ts';
import {
  beforeAfter,
  editorClipWarn,
  editorFullscreen,
  editorInfo,
  floatScope,
  holdOriginal,
  viewerWindow
} from '#lib/commands/editor/view.ts';
import { loupeTrash } from '#lib/commands/loupe/culling.ts';
import { loupeCompare, loupeSingle, loupeSurvey } from '#lib/commands/loupe/navigation.ts';
import {
  loupeAutoAdvance,
  loupeClipWarn,
  loupeFilmstrip,
  loupeFullscreen,
  loupeInfo,
  loupeZoom
} from '#lib/commands/loupe/view.ts';
import type { IconCommand } from '#lib/commands/types.ts';

export type Menu = readonly (readonly IconCommand[])[];

export const LOUPE_VIEW_MODE_MENU: Menu = [[loupeSingle, loupeCompare, loupeSurvey]];

export const LOUPE_MORE_MENU: Menu = [
  [loupeFullscreen, loupeClipWarn, greyCanvas],
  [loupeTrash],
  [help]
];

const LOUPE_PHONE_VIEW: readonly IconCommand[] = [
  loupeZoom,
  loupeFilmstrip,
  loupeFullscreen,
  loupeClipWarn,
  greyCanvas
];

export const LOUPE_PHONE_MORE_MENU: Menu = [
  [loupeAutoAdvance],
  LOUPE_PHONE_VIEW,
  [loupeCompare, loupeSurvey],
  [loupeTrash]
];

export const LOUPE_PHONE_MULTI_MORE_MENU: Menu = [
  [loupeAutoAdvance],
  [loupeInfo, ...LOUPE_PHONE_VIEW],
  [loupeTrash]
];

export const LOUPE_WIDE_MORE_MENU: Menu = [
  [loupeAutoAdvance],
  [loupeFullscreen, loupeClipWarn, greyCanvas],
  [loupeTrash]
];

export const EDITOR_VERSION_MENU: Menu = [[virtualCopy]];

export const EDITOR_VIEW_MENU: Menu = [
  [
    editorInfo,
    editorFullscreen,
    holdOriginal,
    beforeAfter,
    editorClipWarn,
    floatScope,
    greyCanvas,
    viewerWindow
  ]
];

export const EDITOR_MORE_MENU: Menu = [[editorTrash], [help]];

export const EDITOR_PALETTE_MENU: Menu = [
  [
    autoAdjust,
    resetDevelop,
    copyEdits,
    pasteEdits,
    pasteOnly,
    undo,
    redo,
    virtualCopy,
    newSnapshot
  ],
  [
    beforeAfter,
    editorClipWarn,
    floatScope,
    editorInfo,
    editorFullscreen,
    greyCanvas,
    viewerWindow,
    help
  ]
];
