import { greyCanvas, help } from '#lib/commands/app.ts';
import { editorTrash } from '#lib/commands/editor/culling.ts';
import { virtualCopy } from '#lib/commands/editor/edits.ts';
import {
  beforeAfter,
  editorClipWarn,
  editorFullscreen,
  editorInfo,
  holdOriginal
} from '#lib/commands/editor/view.ts';
import { loupeTrash } from '#lib/commands/loupe/culling.ts';
import { loupeCompare, loupeSingle, loupeSurvey } from '#lib/commands/loupe/navigation.ts';
import { loupeClipWarn, loupeFullscreen } from '#lib/commands/loupe/view.ts';
import type { IconCommand } from '#lib/commands/types.ts';

export type Menu = readonly (readonly IconCommand[])[];

export const LOUPE_VIEW_MODE_MENU: Menu = [[loupeSingle, loupeCompare, loupeSurvey]];

export const LOUPE_MORE_MENU: Menu = [
  [loupeFullscreen, loupeClipWarn, greyCanvas],
  [loupeTrash],
  [help]
];

export const EDITOR_VERSION_MENU: Menu = [[virtualCopy]];

export const EDITOR_VIEW_MENU: Menu = [
  [editorInfo, editorFullscreen, holdOriginal, beforeAfter, editorClipWarn, greyCanvas]
];

export const EDITOR_MORE_MENU: Menu = [[editorTrash], [help]];
