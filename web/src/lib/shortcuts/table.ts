import { formatChord, keyLabel } from './chord';

export type KeybindContext =
  | 'global'
  | 'grid'
  | 'loupe'
  | 'compare'
  | 'survey'
  | 'editor'
  | 'geometry'
  | 'masks'
  | 'retouch'
  | 'viewer';

export interface Keybind {
  readonly id: string;
  readonly keys: readonly string[];
  readonly contexts: readonly KeybindContext[];
  readonly group: string;
  readonly label: string;
  readonly display?: string | ((mac: boolean) => string);
  readonly fixed?: boolean;
}

export const KEYBINDS = [
  {
    id: 'help',
    keys: ['?', 'Shift+/'],
    contexts: ['global'],
    group: 'General',
    label: 'Show keyboard shortcuts',
    display: '?'
  },
  {
    id: 'backToGrid',
    keys: ['g'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'General',
    label: 'Back to the grid'
  },

  {
    id: 'rate',
    fixed: true,
    keys: ['0', '1', '2', '3', '4', '5'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Set, toggle or clear the rating',
    display: '0 – 5'
  },
  {
    id: 'rateAdvance',
    fixed: true,
    keys: [
      'Shift+Digit0',
      'Shift+Digit1',
      'Shift+Digit2',
      'Shift+Digit3',
      'Shift+Digit4',
      'Shift+Digit5'
    ],
    contexts: ['grid', 'loupe', 'compare', 'survey'],
    group: 'Culling',
    label: 'Set the rating and move to the next photo',
    display: (mac: boolean) => `${keyLabel('Shift', mac)} 0 – 5`
  },
  {
    id: 'favorite',
    keys: ['p'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Toggle favorite'
  },
  {
    id: 'reject',
    keys: ['x'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Toggle reject'
  },
  {
    id: 'unflag',
    keys: ['u'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Clear favorite and reject'
  },
  {
    id: 'label',
    fixed: true,
    keys: ['6', '7', '8', '9'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Set or clear the red, yellow, green or blue label',
    display: '6 – 9'
  },
  {
    id: 'trash',
    keys: ['Delete', 'Backspace'],
    contexts: ['grid', 'loupe'],
    group: 'Culling',
    label: 'Move to the Immich trash, or delete a virtual copy'
  },

  {
    id: 'gridMove',
    fixed: true,
    keys: ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Select the next photo in that direction'
  },
  {
    id: 'gridExtend',
    fixed: true,
    keys: [
      'Shift+ArrowLeft',
      'Shift+ArrowRight',
      'Shift+ArrowUp',
      'Shift+ArrowDown',
      'Shift+Home',
      'Shift+End',
      'Shift+PageUp',
      'Shift+PageDown'
    ],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Extend the selection, also with Home, End, Page Up and Page Down',
    display: (mac: boolean) =>
      `${keyLabel('Shift', mac)} ${keyLabel('ArrowLeft', mac)} ${keyLabel('ArrowRight', mac)} ${keyLabel('ArrowUp', mac)} ${keyLabel('ArrowDown', mac)}`
  },
  {
    id: 'gridEdge',
    fixed: true,
    keys: ['Home', 'End'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Select the first or last photo'
  },
  {
    id: 'gridPage',
    fixed: true,
    keys: ['PageUp', 'PageDown'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Jump a page'
  },
  {
    id: 'gridSize',
    fixed: true,
    keys: ['-', '_', '=', '+'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Thumbnail size',
    display: '− / +'
  },
  {
    id: 'gridWheelZoom',
    keys: [],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Change the thumbnail size around the pointer',
    display: (mac: boolean) => `${keyLabel('Mod', mac)} scroll · pinch`
  },
  {
    id: 'gridTileInfo',
    keys: ['i'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Cycle thumbnail info: hover only, badges, name and date'
  },
  {
    id: 'toggleFilters',
    keys: ['Shift+f'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Show or hide the filter panel'
  },
  {
    id: 'gridSelectAll',
    keys: ['Mod+a'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Load and select every photo'
  },
  {
    id: 'gridClearSelection',
    fixed: true,
    keys: ['Escape'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Clear the selection'
  },
  {
    id: 'gridMarquee',
    keys: [],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Box-select the photos it touches',
    display: 'Drag from a gap'
  },
  {
    id: 'gridMarqueeAdd',
    keys: [],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Add the boxed photos to the selection',
    display: (mac: boolean) =>
      `${keyLabel('Shift', mac)} or ${keyLabel('Mod', mac)} + drag from a gap`
  },
  {
    id: 'focusBulkBar',
    keys: ['Shift+b'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Focus the selection actions'
  },
  {
    id: 'openLoupe',
    keys: ['e', 'Space'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Open the loupe'
  },
  {
    id: 'peek',
    keys: ['w'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Hold to peek at the photo',
    display: 'W (hold)'
  },
  {
    id: 'openEditor',
    keys: ['d', 'Enter'],
    contexts: ['grid', 'loupe'],
    group: 'Grid',
    label: 'Open the editor'
  },
  {
    id: 'enterCompare',
    keys: ['c'],
    contexts: ['grid', 'loupe'],
    group: 'Grid',
    label: 'Compare the selection, or this photo and the next'
  },
  {
    id: 'enterSurvey',
    keys: ['n'],
    contexts: ['grid', 'loupe'],
    group: 'Grid',
    label: 'Survey the selection, or this photo and the next five'
  },

  {
    id: 'loupeNav',
    fixed: true,
    keys: ['ArrowLeft', 'ArrowRight'],
    contexts: ['loupe'],
    group: 'Loupe',
    label: 'Previous / next photo'
  },
  {
    id: 'zoomPan',
    fixed: true,
    keys: ['Shift+ArrowLeft', 'Shift+ArrowRight', 'Shift+ArrowUp', 'Shift+ArrowDown'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'Loupe',
    label: 'Nudge a zoomed photo'
  },
  {
    id: 'loupeEdge',
    fixed: true,
    keys: ['Home', 'End'],
    contexts: ['loupe'],
    group: 'Loupe',
    label: 'First / last photo'
  },
  {
    id: 'zoomToggle',
    keys: ['z', 'Space'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'Loupe',
    label: 'Toggle zoom'
  },
  {
    id: 'toggleInfo',
    keys: ['i'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'Loupe',
    label: 'Toggle the info panel'
  },
  {
    id: 'toggleTags',
    keys: ['Shift+t'],
    contexts: ['loupe', 'editor'],
    group: 'Loupe',
    label: 'Toggle the tags panel'
  },
  {
    id: 'recentTag',
    fixed: true,
    keys: [
      'Alt+Digit1',
      'Alt+Digit2',
      'Alt+Digit3',
      'Alt+Digit4',
      'Alt+Digit5',
      'Alt+Digit6',
      'Alt+Digit7',
      'Alt+Digit8'
    ],
    contexts: ['loupe', 'compare', 'survey'],
    group: 'Loupe',
    label: 'Add one of the eight recent tags',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} 1 – 8`
  },
  {
    id: 'clipWarn',
    keys: ['j'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'Loupe',
    label: 'Toggle the clipping indicators'
  },
  {
    id: 'loupeClose',
    fixed: true,
    keys: ['Escape'],
    contexts: ['loupe'],
    group: 'Loupe',
    label: 'Close the loupe'
  },

  {
    id: 'compareFocus',
    fixed: true,
    keys: ['ArrowLeft', 'ArrowRight'],
    contexts: ['compare'],
    group: 'Compare',
    label: 'Move focus between panes'
  },
  {
    id: 'paneFocusCycle',
    fixed: true,
    keys: ['Tab', 'Shift+Tab'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Cycle focus between panes'
  },
  {
    id: 'paneSwap',
    fixed: true,
    keys: ['Alt+ArrowLeft', 'Alt+ArrowRight'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Swap the focused pane for another photo'
  },
  {
    id: 'paneSync',
    keys: ['y'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Toggle synced zoom and pan'
  },
  {
    id: 'paneOpenEditor',
    keys: ['d'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Open the focused photo in the editor'
  },
  {
    id: 'paneDrop',
    keys: ['Backspace', 'Delete'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Drop the focused photo'
  },
  {
    id: 'panePromote',
    keys: ['Enter'],
    contexts: ['compare'],
    group: 'Compare',
    label: 'Make the focused photo the select'
  },
  {
    id: 'compareExit',
    fixed: true,
    keys: ['e', 'Escape'],
    contexts: ['compare'],
    group: 'Compare',
    label: 'Back to the loupe on the focused photo'
  },

  {
    id: 'surveyFocus',
    fixed: true,
    keys: ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'],
    contexts: ['survey'],
    group: 'Survey',
    label: 'Move focus between panes'
  },
  {
    id: 'surveyKeep',
    keys: ['Enter'],
    contexts: ['survey'],
    group: 'Survey',
    label: 'Keep only the focused photo'
  },
  {
    id: 'surveyExit',
    fixed: true,
    keys: ['e', 'Escape'],
    contexts: ['survey'],
    group: 'Survey',
    label: 'Back to the loupe, selecting the survivors in the grid if you dropped any'
  },

  {
    id: 'fullscreen',
    keys: ['f'],
    contexts: ['loupe', 'compare', 'survey', 'editor', 'viewer'],
    group: 'View',
    label: 'Toggle fullscreen'
  },
  {
    id: 'filmstrip',
    keys: ['Shift+s'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'View',
    label: 'Hide or show the filmstrip'
  },
  {
    id: 'floatScope',
    keys: ['s'],
    contexts: ['editor'],
    group: 'View',
    label: 'Show or hide the scope overlay'
  },

  {
    id: 'loupeClickZoom',
    keys: [],
    contexts: ['loupe', 'compare', 'survey'],
    group: 'Loupe gestures',
    label: 'Zoom in at the pointer, or back to fit when zoomed',
    display: 'Click the photo'
  },
  {
    id: 'loupeDragPan',
    keys: [],
    contexts: ['loupe', 'compare', 'survey'],
    group: 'Loupe gestures',
    label: 'Pan a zoomed photo',
    display: 'Drag or scroll'
  },
  {
    id: 'loupeWheelZoom',
    keys: [],
    contexts: ['loupe', 'compare', 'survey'],
    group: 'Loupe gestures',
    label: 'Zoom at the pointer',
    display: (mac: boolean) => `${keyLabel('Mod', mac)} + scroll`
  },
  {
    id: 'paneSoloView',
    keys: [],
    contexts: ['compare', 'survey'],
    group: 'Loupe gestures',
    label: 'Zoom or pan only the focused pane while zoom and pan are synced',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} + click or drag`
  },
  {
    id: 'stripTogglePane',
    keys: [],
    contexts: ['loupe', 'compare', 'survey'],
    group: 'Loupe gestures',
    label: 'Add or remove a compare pane',
    display: (mac: boolean) =>
      `${keyLabel('Shift', mac)} or ${keyLabel('Mod', mac)} + click a filmstrip thumbnail`
  },

  {
    id: 'editorNav',
    fixed: true,
    keys: ['ArrowLeft', 'ArrowRight'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Previous / next photo'
  },
  {
    id: 'undo',
    keys: ['Mod+z'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Undo'
  },
  {
    id: 'redo',
    keys: ['Mod+Shift+z'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Redo'
  },
  {
    id: 'openDevelop',
    keys: ['Shift+d'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Develop'
  },
  {
    id: 'openGeometry',
    keys: ['Shift+r'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Geometry'
  },
  {
    id: 'openRetouch',
    keys: ['Shift+q'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Retouch'
  },
  {
    id: 'openMasks',
    keys: ['Shift+m'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Masks'
  },
  {
    id: 'presetStrip',
    keys: ['Shift+p'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Show or hide the presets strip'
  },
  {
    id: 'createVirtualCopy',
    keys: ["Mod+'"],
    contexts: ['grid', 'loupe', 'editor'],
    group: 'Editor',
    label: 'Create a virtual copy'
  },
  {
    id: 'newSnapshot',
    keys: ['Mod+Shift+s'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Save a snapshot of the current edits'
  },
  {
    id: 'beforeAfter',
    keys: ['y'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Cycle compare: split, side by side, off'
  },
  {
    id: 'holdOriginal',
    keys: ['\\', '<'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Hold to view the original',
    display: '\\ or < (hold)'
  },
  {
    id: 'togglePanels',
    keys: ['Tab'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Hide or show the side panels'
  },
  {
    id: 'toggleChrome',
    keys: ['Shift+Tab'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Hide or show every panel'
  },
  {
    id: 'pickWhiteBalance',
    keys: ['w'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Pick white balance from a neutral area'
  },
  {
    id: 'targetAdjust',
    keys: ['t'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Adjust HSL, Curves, B&W or Color Grading on the photo'
  },
  {
    id: 'autoAdjust',
    keys: ['Mod+u'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Auto adjust'
  },
  {
    id: 'searchSettings',
    keys: ['Mod+k'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Search settings, presets and actions'
  },
  {
    id: 'pinSetting',
    keys: ['Shift+Enter'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Pin or unpin the highlighted palette setting'
  },
  {
    id: 'resetEdits',
    keys: ['Mod+Shift+r'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Reset develop settings, or geometry on the Geometry tab'
  },
  {
    id: 'copyEdits',
    keys: ['Mod+c'],
    contexts: ['editor', 'grid', 'loupe'],
    group: 'Editor',
    label: 'Copy edits'
  },
  {
    id: 'pasteEdits',
    keys: ['Mod+v'],
    contexts: ['editor', 'grid', 'loupe'],
    group: 'Editor',
    label: 'Paste edits'
  },
  {
    id: 'pasteOnly',
    keys: ['Mod+Shift+v'],
    contexts: ['editor', 'grid', 'loupe'],
    group: 'Editor',
    label: 'Paste only some settings'
  },
  {
    id: 'openExport',
    keys: ['Shift+e'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Export'
  },
  {
    id: 'editorEscape',
    fixed: true,
    keys: ['Escape'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Step out of the active tool, panel or fullscreen'
  },

  {
    id: 'sliderCoarseStep',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Move a focused slider ten steps',
    display: (mac: boolean) =>
      `${formatChord('Shift+ArrowLeft', mac)} / ${formatChord('Shift+ArrowRight', mac)}`
  },
  {
    id: 'sliderFineStep',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Move a focused slider by its finest step',
    display: (mac: boolean) =>
      `${formatChord('Alt+ArrowLeft', mac)} / ${formatChord('Alt+ArrowRight', mac)}`
  },
  {
    id: 'sliderWheel',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Adjust the slider under the pointer once it has rested there, ten steps or fine',
    display: (mac: boolean) =>
      `Wheel, ${keyLabel('Shift', mac)} + wheel, ${keyLabel('Alt', mac)} + wheel`
  },
  {
    id: 'sliderScrub',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Adjust a slider from its current value; move away from the track to slow down',
    display: 'Drag the label or track'
  },
  {
    id: 'sliderTypedValue',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Type a value, or +5, x1.5 or /2 to change the current one',
    display: 'Click the value'
  },
  {
    id: 'sliderReset',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Reset one slider',
    display: 'Double-click the label'
  },
  {
    id: 'sliderPreviewDrag',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Preview the mask or radius a slider drives while you drag it',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} + drag`
  },
  {
    id: 'sliderResetGroup',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Reset every band or channel at once, on HSL and Curves',
    display: (mac: boolean) => `${keyLabel('Shift', mac)} + click the reset button`
  },
  {
    id: 'curveFineDrag',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Move a curve point at a quarter of the speed',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} + drag a curve point`
  },
  {
    id: 'targetHueDrag',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Adjust hue while dragging on the photo',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} + drag on the photo`
  },
  {
    id: 'targetLightnessDrag',
    keys: [],
    contexts: ['editor'],
    group: 'Sliders',
    label: 'Adjust luminance while dragging on the photo',
    display: (mac: boolean) => `${keyLabel('Shift', mac)} + drag on the photo`
  },

  {
    id: 'straighten',
    keys: ['a'],
    contexts: ['geometry'],
    group: 'Geometry',
    label: 'Draw a line along the horizon to level the photo'
  },
  {
    id: 'straightenDrag',
    keys: [],
    contexts: ['geometry'],
    group: 'Geometry',
    label: 'Draw a straighten line without arming the tool',
    display: (mac: boolean) => `${keyLabel('Shift', mac)} + drag inside the crop`
  },
  {
    id: 'cropGrid',
    keys: ['o'],
    contexts: ['geometry'],
    group: 'Geometry',
    label: 'Cycle the crop guide: thirds, golden ratio, diagonals, grid, none'
  },
  {
    id: 'cropFromCentre',
    keys: [],
    contexts: ['geometry'],
    group: 'Geometry',
    label: 'Resize the crop about its centre',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} + drag a crop handle`
  },
  {
    id: 'cropLockRatio',
    keys: [],
    contexts: ['geometry'],
    group: 'Geometry',
    label: 'Keep the current shape of a free crop while resizing',
    display: (mac: boolean) => `${keyLabel('Shift', mac)} + drag a crop handle`
  },
  {
    id: 'cropScale',
    keys: [],
    contexts: ['geometry'],
    group: 'Geometry',
    label: 'Scale the crop about its centre',
    display: (mac: boolean) => `${keyLabel('Mod', mac)} + scroll over the crop`
  },
  {
    id: 'cropPan',
    keys: [],
    contexts: ['geometry'],
    group: 'Geometry',
    label: 'Move the photo under a fixed crop',
    display: (mac: boolean) => `${keyLabel('Mod', mac)} + drag inside the crop`
  },

  {
    id: 'maskDelete',
    keys: ['Backspace', 'Delete'],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Delete the selected shape, or undo the last polygon corner'
  },
  {
    id: 'addBrushLayer',
    keys: ['k'],
    contexts: ['editor'],
    group: 'Masks',
    label: 'Add a brush mask layer'
  },
  {
    id: 'addRadialLayer',
    keys: ['r'],
    contexts: ['editor'],
    group: 'Masks',
    label: 'Add a radial mask layer'
  },
  {
    id: 'addLinearLayer',
    keys: ['l'],
    contexts: ['editor'],
    group: 'Masks',
    label: 'Add a linear mask layer'
  },
  {
    id: 'maskNudge',
    fixed: true,
    keys: [
      'ArrowUp',
      'ArrowDown',
      'Shift+ArrowLeft',
      'Shift+ArrowRight',
      'Shift+ArrowUp',
      'Shift+ArrowDown'
    ],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Nudge the selected shape one pixel, ten with Shift',
    display: (mac: boolean) =>
      `${keyLabel('ArrowLeft', mac)} ${keyLabel('ArrowRight', mac)} ${keyLabel('ArrowUp', mac)} ${keyLabel('ArrowDown', mac)}`
  },
  {
    id: 'polygonAddCorner',
    keys: [],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Add a polygon corner',
    display: 'Click the dot between two corners'
  },
  {
    id: 'polygonRemoveCorner',
    keys: [],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Remove a polygon corner',
    display: 'Double-click the corner'
  },
  {
    id: 'maskRoundRadial',
    keys: [],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Keep a radial round while resizing it',
    display: (mac: boolean) => `${keyLabel('Shift', mac)} + drag a radial size handle`
  },
  {
    id: 'maskSnapLinear',
    keys: [],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Snap a linear gradient to 45° steps',
    display: (mac: boolean) => `${keyLabel('Shift', mac)} + drag a linear end`
  },
  {
    id: 'maskLockCorner',
    keys: [],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Move a polygon corner straight across or straight up and down',
    display: (mac: boolean) => `${keyLabel('Shift', mac)} + drag a polygon corner`
  },
  {
    id: 'maskOverlay',
    keys: ['o'],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Toggle the mask overlay'
  },
  {
    id: 'maskCancelDraw',
    fixed: true,
    keys: ['Escape'],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Cancel drawing, box select or the eyedropper'
  },
  {
    id: 'maskClosePolygon',
    keys: ['Enter'],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Close the polygon you are drawing'
  },
  {
    id: 'brushSize',
    fixed: true,
    keys: ['[', ']'],
    contexts: ['masks', 'retouch'],
    group: 'Masks',
    label: 'Smaller / larger brush'
  },
  {
    id: 'brushHardness',
    fixed: true,
    keys: ['{', '}'],
    contexts: ['masks', 'retouch'],
    group: 'Masks',
    label: 'Softer / harder brush'
  },
  {
    id: 'brushSizeWheel',
    keys: [],
    contexts: ['masks', 'retouch'],
    group: 'Masks',
    label: 'Resize the brush over the photo',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} + scroll`
  },
  {
    id: 'brushHardnessWheel',
    keys: [],
    contexts: ['masks', 'retouch'],
    group: 'Masks',
    label: 'Change the brush hardness over the photo',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} + ${keyLabel('Shift', mac)} + scroll`
  },

  {
    id: 'retouchHeal',
    keys: ['h'],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Heal mode'
  },
  {
    id: 'retouchClone',
    keys: ['c'],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Clone mode'
  },
  {
    id: 'retouchDelete',
    keys: ['Backspace', 'Delete'],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Delete the selected stroke'
  },
  {
    id: 'retouchDeselect',
    keys: [],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Deselect the current stroke',
    display: (mac: boolean) => keyLabel('Escape', mac)
  },
  {
    id: 'retouchSample',
    keys: [],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Set the source point',
    display: (mac: boolean) => `${keyLabel('Alt', mac)} + click`
  },
  {
    id: 'retouchMoveSource',
    keys: [],
    contexts: ['retouch'],
    group: 'Retouch',
    label: "Move the selected stroke's source",
    display: 'Drag the green ring'
  }
] as const satisfies readonly Keybind[];

export type KeybindId = (typeof KEYBINDS)[number]['id'];
