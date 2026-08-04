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
  { id: '7:5', label: '7:5', value: 7 / 5 },
];

class UiStore {
  leftCollapsed = $state(false);
  rightCollapsed = $state(false);
  filmstripCollapsed = $state(false);
  searchQuery = $state('');
  fullscreen = $state(false);
  zoom = $state(100);
  panX = $state(0);
  panY = $state(0);
  keybindsHelpOpen = $state(false);
  exifPopoverOpen = $state(false);
  tagsPopoverOpen = $state(false);
  zoomPopoverOpen = $state(false);
  editorTab = $state<EditorTab>('develop');
  perspectiveCorners = $state(false);

  togglePerspectiveCorners = (): void => {
    this.perspectiveCorners = !this.perspectiveCorners;
  };

  toggleLeft = (): void => {
    this.leftCollapsed = !this.leftCollapsed;
  };

  toggleRight = (): void => {
    this.rightCollapsed = !this.rightCollapsed;
  };

  openGeometry = (): void => {
    this.fullscreen = false;
    this.rightCollapsed = false;
    this.editorTab = 'geometry';
  };

  openRetouch = (): void => {
    this.fullscreen = false;
    this.rightCollapsed = false;
    this.editorTab = 'retouch';
  };

  toggleFilmstrip = (): void => {
    this.filmstripCollapsed = !this.filmstripCollapsed;
  };

  toggleFullscreen = (): void => {
    this.fullscreen = !this.fullscreen;
  };

  zoomIn = (): void => {
    this.zoom = Math.min(this.zoom + 25, 400);
  };

  zoomOut = (): void => {
    this.zoom = Math.max(this.zoom - 25, 25);
    if (this.zoom <= 100) { this.panX = 0; this.panY = 0; }
  };

  zoomFit = (): void => {
    this.zoom = 100;
    this.panX = 0;
    this.panY = 0;
  };

  zoomToggle = (): void => {
    if (this.zoom <= 100) {
      this.zoom = 200;
    } else {
      this.zoomFit();
    }
  };

  toggleKeybindsHelp = (): void => {
    this.keybindsHelpOpen = !this.keybindsHelpOpen;
  };

  closeKeybindsHelp = (): void => {
    this.keybindsHelpOpen = false;
  };

  openExifPopover = (): void => {
    this.tagsPopoverOpen = false;
    this.zoomPopoverOpen = false;
    this.exifPopoverOpen = true;
  };

  toggleExifPopover = (): void => {
    if (this.exifPopoverOpen) {
      this.exifPopoverOpen = false;
    } else {
      this.openExifPopover();
    }
  };

  closeExifPopover = (): void => {
    this.exifPopoverOpen = false;
  };

  openTagsPopover = (): void => {
    this.exifPopoverOpen = false;
    this.zoomPopoverOpen = false;
    this.tagsPopoverOpen = true;
  };

  toggleTagsPopover = (): void => {
    if (this.tagsPopoverOpen) {
      this.tagsPopoverOpen = false;
    } else {
      this.openTagsPopover();
    }
  };

  closeTagsPopover = (): void => {
    this.tagsPopoverOpen = false;
  };

  closeMetadataPopovers = (): boolean => {
    if (!this.exifPopoverOpen && !this.tagsPopoverOpen && !this.zoomPopoverOpen) return false;
    this.exifPopoverOpen = false;
    this.tagsPopoverOpen = false;
    this.zoomPopoverOpen = false;
    return true;
  };

  openZoomPopover = (): void => {
    this.exifPopoverOpen = false;
    this.tagsPopoverOpen = false;
    this.zoomPopoverOpen = true;
  };

  toggleZoomPopover = (): void => {
    if (this.zoomPopoverOpen) {
      this.zoomPopoverOpen = false;
    } else {
      this.openZoomPopover();
    }
  };

  closeZoomPopover = (): void => {
    this.zoomPopoverOpen = false;
  };

  setZoom = (value: number): void => {
    this.zoom = Math.round(Math.max(25, Math.min(400, value)));
    if (this.zoom <= 100) { this.panX = 0; this.panY = 0; }
  };
}

export const ui = new UiStore();
