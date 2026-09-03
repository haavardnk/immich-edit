import { readStored, writeStored } from '$lib/utils/storage';
import { clampZoom, nextStop, readZoomLevel, writeZoomLevel } from '$lib/utils/zoomLevel';

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

type PersistedEditorUi = {
  inspectorWidth?: number;
  filmstripHeight?: number;
  rightCollapsed?: boolean;
  editorFilmstripCollapsed?: boolean;
  loupeFilmstripCollapsed?: boolean;
};

export type MetaPopover = 'exif' | 'tags' | 'zoom';

class UiStore {
  rightCollapsed = $state(false);
  inspectorWidth = $state(384);
  filmstripHeight = $state(64);
  editorFilmstripCollapsed = $state(false);
  loupeFilmstripCollapsed = $state(false);
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
  clipWarn = $state(false);

  constructor() {
    const stored = readStored<PersistedEditorUi>(UI_STORAGE_KEY);
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
  }

  persistEditorUi = (): void => {
    writeStored(UI_STORAGE_KEY, {
      inspectorWidth: this.inspectorWidth,
      filmstripHeight: this.filmstripHeight,
      rightCollapsed: this.rightCollapsed,
      editorFilmstripCollapsed: this.editorFilmstripCollapsed,
      loupeFilmstripCollapsed: this.loupeFilmstripCollapsed
    } satisfies PersistedEditorUi);
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

  togglePerspectiveCorners = (): void => {
    this.perspectiveCorners = !this.perspectiveCorners;
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
