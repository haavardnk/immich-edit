import {
  BRUSH_FLOW,
  BRUSH_HARDNESS,
  BRUSH_SIZE,
  RETOUCH_HARDNESS,
  RETOUCH_OPACITY,
  RETOUCH_SIZE
} from '#lib/editor/params/brush.ts';
import { GUIDE_TURNS } from '#lib/editor/cropGuides.ts';
import type { ToolRange } from '#lib/editor/params/types.ts';
import type { RetouchMode } from '#lib/types/edits.ts';
import { withZoom, type View, type ViewChange, type ZoomedView } from '#lib/utils/imageViewport.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';
import {
  clampZoom,
  isFitZoom,
  nextStop,
  readZoomLevel,
  writeZoomLevel
} from '#lib/utils/zoomLevel.ts';

export type EditorTab = 'develop' | 'masks' | 'retouch' | 'geometry' | 'export';

export type CropGrid = 'thirds' | 'golden' | 'spiral' | 'diagonal' | 'triangle' | 'grid' | 'off';

export type GeometryTool = 'straighten' | 'upright' | 'corners';

export const CROP_GRIDS: { id: CropGrid; label: string }[] = [
  { id: 'thirds', label: 'Rule of thirds' },
  { id: 'golden', label: 'Golden ratio' },
  { id: 'spiral', label: 'Golden spiral' },
  { id: 'diagonal', label: 'Diagonals' },
  { id: 'triangle', label: 'Golden triangle' },
  { id: 'grid', label: 'Grid' },
  { id: 'off', label: 'No guide' }
];

export const MIN_INSPECTOR_WIDTH = 320;
export const MAX_INSPECTOR_WIDTH = 520;
export const MIN_FILMSTRIP_HEIGHT = 48;
export const MAX_FILMSTRIP_HEIGHT = 144;

const UI_STORAGE_KEY = 'immich-edit:editorUi';

export interface BrushTool {
  size: number;
  hardness: number;
  flow: number;
  mode: 'paint' | 'erase';
}

export interface RetouchTool {
  mode: RetouchMode;
  size: number;
  hardness: number;
  opacity: number;
}

const DEFAULT_BRUSH_TOOL: BrushTool = {
  size: BRUSH_SIZE.defaultValue,
  hardness: BRUSH_HARDNESS.defaultValue,
  flow: BRUSH_FLOW.defaultValue,
  mode: 'paint'
};
const DEFAULT_RETOUCH_TOOL: RetouchTool = {
  mode: 'heal',
  size: RETOUCH_SIZE.defaultValue,
  hardness: RETOUCH_HARDNESS.defaultValue,
  opacity: RETOUCH_OPACITY.defaultValue
};

function clampStored(value: unknown, { min, max, defaultValue }: ToolRange): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) return defaultValue;
  return Math.min(max, Math.max(min, value));
}

function restoreBrushTool(
  stored: Partial<Record<keyof BrushTool, unknown>> | undefined
): BrushTool {
  return {
    size: clampStored(stored?.size, BRUSH_SIZE),
    hardness: clampStored(stored?.hardness, BRUSH_HARDNESS),
    flow: clampStored(stored?.flow, BRUSH_FLOW),
    mode: stored?.mode === 'erase' ? 'erase' : 'paint'
  };
}

function restoreRetouchTool(
  stored: Partial<Record<keyof RetouchTool, unknown>> | undefined
): RetouchTool {
  return {
    mode: stored?.mode === 'clone' ? 'clone' : 'heal',
    size: clampStored(stored?.size, RETOUCH_SIZE),
    hardness: clampStored(stored?.hardness, RETOUCH_HARDNESS),
    opacity: clampStored(stored?.opacity, RETOUCH_OPACITY)
  };
}

type PersistedEditorUi = {
  inspectorWidth?: number;
  filmstripHeight?: number;
  rightCollapsed?: boolean;
  editorFilmstripCollapsed?: boolean;
  loupeFilmstripCollapsed?: boolean;
  loupeCompactFilmstrip?: boolean;
  developOpenPanels?: string[];
  brushTool?: BrushTool;
  retouchTool?: RetouchTool;
  greyCanvas?: boolean;
  cropGrid?: CropGrid;
  cropGuideTurn?: number;
};

export type MetaPopover = 'exif' | 'tags' | 'zoom';

class UiStore {
  rightCollapsed = $state(false);
  inspectorWidth = $state(384);
  filmstripHeight = $state(64);
  editorFilmstripCollapsed = $state(false);
  presetStrip = $state(false);
  loupeFilmstripCollapsed = $state(false);
  loupeCompactFilmstrip = $state(false);
  developOpenPanels = $state<string[] | null>(null);
  developModifiedOnly = $state(false);
  searchQuery = $state('');
  fullscreen = $state(false);
  zoom = $state(100);
  fitZoom = $state(100);
  fitMode = $state(true);
  zoomLevel = $state(readZoomLevel());
  cx = $state(0.5);
  cy = $state(0.5);
  keybindsHelpOpen = $state(false);
  metaPopover = $state<MetaPopover | null>(null);
  editorTab = $state<EditorTab>('develop');
  geometryTool = $state<GeometryTool | null>(null);
  cropGrid = $state<CropGrid>('thirds');
  cropGuideTurn = $state(0);
  clipWarn = $state(false);
  greyCanvas = $state(false);
  brushTool = $state<BrushTool>(DEFAULT_BRUSH_TOOL);
  retouchTool = $state<RetouchTool>(DEFAULT_RETOUCH_TOOL);

  constructor() {
    const stored = readStored<PersistedEditorUi>(UI_STORAGE_KEY);
    this.brushTool = restoreBrushTool(stored?.brushTool);
    this.retouchTool = restoreRetouchTool(stored?.retouchTool);
    if (typeof stored?.inspectorWidth === 'number') {
      this.inspectorWidth = Math.min(
        MAX_INSPECTOR_WIDTH,
        Math.max(MIN_INSPECTOR_WIDTH, stored.inspectorWidth)
      );
    }
    if (typeof stored?.filmstripHeight === 'number') {
      this.filmstripHeight = Math.min(
        MAX_FILMSTRIP_HEIGHT,
        Math.max(MIN_FILMSTRIP_HEIGHT, stored.filmstripHeight)
      );
    }
    if (typeof stored?.rightCollapsed === 'boolean') {
      this.rightCollapsed = stored.rightCollapsed;
    }
    if (typeof stored?.editorFilmstripCollapsed === 'boolean') {
      this.editorFilmstripCollapsed = stored.editorFilmstripCollapsed;
    }
    if (typeof stored?.loupeFilmstripCollapsed === 'boolean') {
      this.loupeFilmstripCollapsed = stored.loupeFilmstripCollapsed;
    }
    this.loupeCompactFilmstrip = stored?.loupeCompactFilmstrip === true;
    if (Array.isArray(stored?.developOpenPanels)) {
      this.developOpenPanels = stored.developOpenPanels.filter((id) => typeof id === 'string');
    }
    this.greyCanvas = stored?.greyCanvas === true;
    this.cropGrid = CROP_GRIDS.find((g) => g.id === stored?.cropGrid)?.id ?? 'thirds';
    const turn = stored?.cropGuideTurn;
    this.cropGuideTurn =
      typeof turn === 'number' && Number.isInteger(turn)
        ? ((turn % GUIDE_TURNS) + GUIDE_TURNS) % GUIDE_TURNS
        : 0;
  }

  setDevelopPanels = (ids: string[]): void => {
    this.developOpenPanels = ids;
    this.persistEditorUi();
  };

  persistEditorUi = (): void => {
    writeStored(UI_STORAGE_KEY, {
      inspectorWidth: this.inspectorWidth,
      filmstripHeight: this.filmstripHeight,
      rightCollapsed: this.rightCollapsed,
      editorFilmstripCollapsed: this.editorFilmstripCollapsed,
      loupeFilmstripCollapsed: this.loupeFilmstripCollapsed,
      loupeCompactFilmstrip: this.loupeCompactFilmstrip,
      developOpenPanels: this.developOpenPanels ?? undefined,
      brushTool: this.brushTool,
      retouchTool: this.retouchTool,
      greyCanvas: this.greyCanvas,
      cropGrid: this.cropGrid,
      cropGuideTurn: this.cropGuideTurn
    } satisfies PersistedEditorUi);
  };

  setBrushTool = (tool: BrushTool): void => {
    this.brushTool = tool;
    this.persistEditorUi();
  };

  setRetouchTool = (tool: RetouchTool): void => {
    this.retouchTool = tool;
    this.persistEditorUi();
  };

  setInspectorWidth = (width: number): void => {
    this.inspectorWidth = Math.round(
      Math.min(MAX_INSPECTOR_WIDTH, Math.max(MIN_INSPECTOR_WIDTH, width))
    );
  };

  setFilmstripHeight = (height: number): void => {
    this.filmstripHeight = Math.round(
      Math.min(MAX_FILMSTRIP_HEIGHT, Math.max(MIN_FILMSTRIP_HEIGHT, height))
    );
  };

  toggleClipWarn = (): void => {
    this.clipWarn = !this.clipWarn;
  };

  toggleGreyCanvas = (): void => {
    this.greyCanvas = !this.greyCanvas;
    this.persistEditorUi();
  };

  toggleGeometryTool = (tool: GeometryTool): void => {
    this.geometryTool = this.geometryTool === tool ? null : tool;
  };

  cycleCropGrid = (): void => {
    const at = CROP_GRIDS.findIndex((g) => g.id === this.cropGrid);
    this.cropGrid = CROP_GRIDS[(at + 1) % CROP_GRIDS.length]?.id ?? 'thirds';
    this.persistEditorUi();
  };

  turnCropGuide = (): void => {
    this.cropGuideTurn = (this.cropGuideTurn + 1) % GUIDE_TURNS;
    this.persistEditorUi();
  };

  openTab = (tab: EditorTab): void => {
    this.fullscreen = false;
    this.rightCollapsed = false;
    this.editorTab = tab;
    this.persistEditorUi();
  };

  toggleChrome = (): void => {
    const hidden = this.rightCollapsed && this.editorFilmstripCollapsed && !this.presetStrip;
    this.rightCollapsed = !hidden;
    this.editorFilmstripCollapsed = !hidden;
    this.presetStrip = false;
    this.persistEditorUi();
  };

  togglePanels = (): void => {
    this.rightCollapsed = !this.rightCollapsed;
    this.persistEditorUi();
  };

  toggleEditorFilmstrip = (): void => {
    if (this.presetStrip) {
      this.presetStrip = false;
      this.editorFilmstripCollapsed = false;
    } else {
      this.editorFilmstripCollapsed = !this.editorFilmstripCollapsed;
    }
    this.persistEditorUi();
  };

  togglePresetStrip = (): void => {
    this.presetStrip = !this.presetStrip;
    if (this.presetStrip) return;
    this.editorFilmstripCollapsed = true;
    this.persistEditorUi();
  };

  toggleLoupeFilmstrip = (compact = false): void => {
    if (compact) this.loupeCompactFilmstrip = !this.loupeCompactFilmstrip;
    else this.loupeFilmstripCollapsed = !this.loupeFilmstripCollapsed;
    this.persistEditorUi();
  };

  toggleFullscreen = (): void => {
    this.fullscreen = !this.fullscreen;
  };

  zoomIn = (): void => {
    this.zoomStep(1);
  };

  zoomOut = (): void => {
    this.zoomStep(-1);
  };

  private zoomStep(direction: 1 | -1): void {
    const next = nextStop(this.zoom, direction, this.fitZoom);
    if (isFitZoom(next, this.fitZoom)) this.zoomFit();
    else this.userZoom(next);
  }

  zoomFit = (): void => {
    this.fitMode = true;
    this.zoom = this.fitZoom;
    this.centre();
  };

  toggleKeybindsHelp = (): void => {
    this.keybindsHelpOpen = !this.keybindsHelpOpen;
  };

  closeKeybindsHelp = (): void => {
    this.keybindsHelpOpen = false;
  };

  openPopover = (which: MetaPopover): void => {
    this.metaPopover = which;
  };

  togglePopover = (which: MetaPopover): void => {
    this.metaPopover = this.metaPopover === which ? null : which;
  };

  closePopover = (): void => {
    this.metaPopover = null;
  };

  get zoomed(): boolean {
    return this.zoom > this.fitZoom;
  }

  get view(): View {
    return { zoom: this.fitMode ? null : this.zoom, cx: this.cx, cy: this.cy };
  }

  setFitZoom = (value: number): void => {
    const next = Number.isFinite(value) && value > 0 ? value : 100;
    if (this.fitZoom === next) return;
    this.fitZoom = next;
    if (this.fitMode) {
      this.zoom = next;
      return;
    }
    this.zoom = clampZoom(this.zoom, next);
  };

  setZoom = (value: number): void => {
    this.show(withZoom(this.view, value, this.fitZoom));
  };

  userZoom = (value: number): void => {
    this.setZoom(value);
    this.rememberZoomLevel(this.zoom);
  };

  setView = (view: View): void => {
    if (view.zoom === null) {
      this.zoomFit();
      return;
    }
    this.show(withZoom(view, view.zoom, this.fitZoom));
  };

  applyView = (view: View, change: ViewChange): void => {
    this.setView(view);
    if (change === 'zoom') this.rememberZoomLevel(this.zoom);
  };

  rememberZoomLevel = (zoom: number, fitZoom: number = this.fitZoom): void => {
    if (zoom <= fitZoom) return;
    this.zoomLevel = zoom;
    writeZoomLevel(zoom);
  };

  private show(view: ZoomedView): void {
    this.fitMode = false;
    this.zoom = view.zoom;
    this.cx = view.cx;
    this.cy = view.cy;
  }

  private centre(): void {
    this.cx = 0.5;
    this.cy = 0.5;
  }

  zoomNative = (): void => {
    this.userZoom(100);
  };
}

export const ui = new UiStore();
