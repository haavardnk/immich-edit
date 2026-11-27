import { APP_COMMANDS } from '#lib/commands/app.ts';
import { CULLING_COMMANDS } from '#lib/commands/editor/culling.ts';
import { DISMISS_COMMANDS } from '#lib/commands/editor/dismiss.ts';
import { EDIT_COMMANDS } from '#lib/commands/editor/edits.ts';
import { NAVIGATION_COMMANDS } from '#lib/commands/editor/navigation.ts';
import { TOOL_COMMANDS } from '#lib/commands/editor/tools.ts';
import { VIEW_COMMANDS } from '#lib/commands/editor/view.ts';
import { GRID_CULLING_COMMANDS } from '#lib/commands/grid/culling.ts';
import { GRID_EDIT_COMMANDS } from '#lib/commands/grid/edits.ts';
import { GRID_EXPORT_COMMANDS } from '#lib/commands/grid/export.ts';
import { GRID_FILTER_COMMANDS } from '#lib/commands/grid/filters.ts';
import { GRID_LOCATION_COMMANDS } from '#lib/commands/grid/location.ts';
import { GRID_NAVIGATION_COMMANDS } from '#lib/commands/grid/navigation.ts';
import { GRID_SELECTION_COMMANDS } from '#lib/commands/grid/selection.ts';
import { GRID_VIEW_COMMANDS } from '#lib/commands/grid/view.ts';
import { createKeyDispatcher } from '#lib/commands/keys.ts';
import { LOUPE_ALBUM_COMMANDS } from '#lib/commands/loupe/album.ts';
import { LOUPE_CULLING_COMMANDS } from '#lib/commands/loupe/culling.ts';
import { LOUPE_DISMISS_COMMANDS } from '#lib/commands/loupe/dismiss.ts';
import { LOUPE_EDIT_COMMANDS } from '#lib/commands/loupe/edits.ts';
import { LOUPE_LOCATION_COMMANDS } from '#lib/commands/loupe/location.ts';
import { LOUPE_NAVIGATION_COMMANDS } from '#lib/commands/loupe/navigation.ts';
import { LOUPE_PANE_COMMANDS } from '#lib/commands/loupe/panes.ts';
import { LOUPE_VIEW_COMMANDS } from '#lib/commands/loupe/view.ts';
import { MAP_COMMANDS } from '#lib/commands/map.ts';
import { SETTINGS_COMMANDS } from '#lib/commands/settings.ts';
import type { Command } from '#lib/commands/types.ts';
import { VIEWER_COMMANDS } from '#lib/commands/viewer.ts';

export const COMMANDS: readonly Command[] = [
  ...APP_COMMANDS,
  ...DISMISS_COMMANDS,
  ...VIEW_COMMANDS,
  ...NAVIGATION_COMMANDS,
  ...EDIT_COMMANDS,
  ...CULLING_COMMANDS,
  ...TOOL_COMMANDS,
  ...LOUPE_DISMISS_COMMANDS,
  ...LOUPE_VIEW_COMMANDS,
  ...LOUPE_NAVIGATION_COMMANDS,
  ...LOUPE_PANE_COMMANDS,
  ...LOUPE_EDIT_COMMANDS,
  ...LOUPE_LOCATION_COMMANDS,
  ...LOUPE_CULLING_COMMANDS,
  ...LOUPE_ALBUM_COMMANDS,
  ...MAP_COMMANDS,
  ...GRID_VIEW_COMMANDS,
  ...GRID_SELECTION_COMMANDS,
  ...GRID_NAVIGATION_COMMANDS,
  ...GRID_EDIT_COMMANDS,
  ...GRID_LOCATION_COMMANDS,
  ...GRID_EXPORT_COMMANDS,
  ...GRID_CULLING_COMMANDS,
  ...GRID_FILTER_COMMANDS,
  ...VIEWER_COMMANDS,
  ...SETTINGS_COMMANDS
];

export const commandKeys = createKeyDispatcher(COMMANDS);
