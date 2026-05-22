export type AspectRatio = 'free' | 'original' | '1:1' | '4:3' | '3:2' | '16:9' | '5:4' | '7:5';

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

  toggleLeft = (): void => {
    this.leftCollapsed = !this.leftCollapsed;
  };

  toggleRight = (): void => {
    this.rightCollapsed = !this.rightCollapsed;
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

  setZoom = (value: number): void => {
    this.zoom = Math.round(Math.max(25, Math.min(400, value)));
    if (this.zoom <= 100) { this.panX = 0; this.panY = 0; }
  };
}

export const ui = new UiStore();
