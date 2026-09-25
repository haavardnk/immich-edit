import { readStored, writeStored } from '$lib/utils/storage';
import { clampZoom, nextStop, readZoomLevel, writeZoomLevel } from '$lib/utils/zoomLevel';
import type { RetouchMode } from '$lib/types/edits';

export type AspectRatio = 'free' | 'original' | '1:1' | '4:3' | '3:2' | '16:9' | '5:4' | '7:5';
export type EditorTab = 'develop' | 'masks' | 'retouch' | 'geometry' | 'export';

export const ASPECT_RATIOS: { id: AspectRatio; label: string; value: number | null }[] = [
  { id: 'free', label: 'Free', value: null },
  { id: 'original', label: 'Original', value: null },
  { id: '1:1', label: '1:1', value: 1 },
  { id: '4:3', label: '4:3', value: 4 / 3 },
  { id: '3:2', label: '3:2', value: 3 / 2 },
  { id: '16:9', label: '16:9', value: 16 / 9 },
  { id: '5:4', label: '5:4', value: 5 / 4 },
  { id: '7:5', label: '7:5', value: 7 / 5 }
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

const DEFAULT_BRUSH_TOOL: BrushTool = { size: 0.08, hardness: 0.5, flow: 0.8, mode: 'paint' };
const DEFAULT_RETOUCH_TOOL: RetouchTool = { mode: 'heal', size: 0.05, hardness: 0.5, opacity: 1 };

function clampStored(value: unknown, min: number, max: number, fallback: number): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) return fallback;
  return Math.min(max, Math.max(min, value));
}

function restoreBrushTool(
  stored: Partial<Record<keyof BrushTool, unknown>> | undefined
): BrushTool {
  const d = DEFAULT_BRUSH_TOOL;
  return {
    size: clampStored(stored?.size, 0.005, 0.5, d.size),
    hardness: clampStored(stored?.hardness, 0, 1, d.hardness),
    flow: clampStored(stored?.flow, 0.01, 1, d.flow),
    mode: stored?.mode === 'erase' ? 'erase' : 'paint'
  };
}

function restoreRetouchTool(
  stored: Partial<Record<keyof RetouchTool, unknown>> | undefined
): RetouchTool {
  const d = DEFAULT_RETOUCH_TOOL;
  return {
    mode: stored?.mode === 'clone' ? 'clone' : 'heal',
    size: clampStored(stored?.size, 0.005, 0.3, d.size),
    hardness: clampStored(stored?.hardness, 0, 1, d.hardness),
    opacity: clampStored(stored?.opacity, 0.05, 1, d.opacity)
  };
}

type PersistedEditorUi = {
  inspectorWidth?: number;
  filmstripHeight?: number;
  rightCollapsed?: boolean;
  editorFilmstripCollapsed?: boolean;
  loupeFilmstripCollapsed?: boolean;
  developOpenPanels?: string[];
  brushTool?: BrushTool;
  retouchTool?: RetouchTool;
  greyCanvas?: boolean;
};

export type MetaPopover = 'exif' | 'tags' | 'zoom';

class UiStore {
  rightCollapsed = $state(false);
  inspectorWidth = $state(384);
  filmstripHeight = $state(64);
  editorFilmstripCollapsed = $state(false);
  loupeFilmstripCollapsed = $state(false);
  developOpenPanels = $state<string[] | null>(null);
  developModifiedOnly = $state(false);
  searchQuery = $state('');
  fullscreen = $state(false);
  zoom = $state(100);
  fitZoom = $state(100);
  fitMode = $state(true);
  zoomLevel = $state(readZoomLevel());
  panX = $state(0);
  panY = $state(0);
  keybindsHelpOpen = $state(false);
  metaPopover = $state<MetaPopover | null>(null);
  editorTab = $state<EditorTab>('develop');
  perspectiveCorners = $state(false);
  straightening = $state(false);
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
    if (Array.isArray(stored?.developOpenPanels)) {
      this.developOpenPanels = stored.developOpenPanels.filter((id) => typeof id === 'string');
    }
    this.greyCanvas = stored?.greyCanvas === true;
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
      developOpenPanels: this.developOpenPanels ?? undefined,
      brushTool: this.brushTool,
      retouchTool: this.retouchTool,
      greyCanvas: this.greyCanvas
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

  togglePerspectiveCorners = (): void => {
    this.perspectiveCorners = !this.perspectiveCorners;
  };

  toggleStraighten = (): void => {
    this.straightening = !this.straightening;
  };

  openTab = (tab: EditorTab): void => {
    this.fullscreen = false;
    this.rightCollapsed = false;
    this.editorTab = tab;
    this.persistEditorUi();
  };

  toggleChrome = (): void => {
    const hidden = this.rightCollapsed && this.editorFilmstripCollapsed;
    this.rightCollapsed = !hidden;
    this.editorFilmstripCollapsed = !hidden;
    this.persistEditorUi();
  };

  togglePanels = (): void => {
    this.rightCollapsed = !this.rightCollapsed;
    this.persistEditorUi();
  };

  toggleEditorFilmstrip = (): void => {
    this.editorFilmstripCollapsed = !this.editorFilmstripCollapsed;
    this.persistEditorUi();
  };

  toggleLoupeFilmstrip = (): void => {
    this.loupeFilmstripCollapsed = !this.loupeFilmstripCollapsed;
    this.persistEditorUi();
  };

  toggleFullscreen = (): void => {
    this.fullscreen = !this.fullscreen;
  };

  zoomIn = (): void => {
    this.userZoom(nextStop(this.zoom, 1, this.fitZoom));
  };

  zoomOut = (): void => {
    this.userZoom(nextStop(this.zoom, -1, this.fitZoom));
  };

  zoomFit = (): void => {
    this.fitMode = true;
    this.zoom = this.fitZoom;
    this.panX = 0;
    this.panY = 0;
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

  closeMetadataPopovers = (): boolean => {
    if (!this.metaPopover) return false;
    this.metaPopover = null;
    return true;
  };

  get zoomed(): boolean {
    return this.zoom > this.fitZoom;
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
    this.fitMode = false;
    this.zoom = clampZoom(value, this.fitZoom);
    if (!this.zoomed) {
      this.panX = 0;
      this.panY = 0;
    }
  };

  userZoom = (value: number): void => {
    this.setZoom(value);
    if (!this.zoomed) return;
    this.zoomLevel = this.zoom;
    writeZoomLevel(this.zoom);
  };

  setView = (zoom: number, panX: number, panY: number): void => {
    this.setZoom(zoom);
    if (!this.zoomed) return;
    this.panX = panX;
    this.panY = panY;
  };

  zoomNative = (): void => {
    this.userZoom(100);
  };
}

export const ui = new UiStore();
