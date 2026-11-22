import { readStored, writeStored } from '#lib/utils/storage.ts';

const KEY = 'immich-edit:mapPanel';

export const MIN_MAP_PANEL = 280;
export const MAX_MAP_PANEL = 640;
export const DEFAULT_MAP_PANEL = 360;

export function clampPanel(width: number): number {
  if (!Number.isFinite(width)) return DEFAULT_MAP_PANEL;
  return Math.round(Math.min(MAX_MAP_PANEL, Math.max(MIN_MAP_PANEL, width)));
}

class MapPanel {
  width = $state(clampPanel(readStored<{ width: number }>(KEY)?.width ?? DEFAULT_MAP_PANEL));

  resize = (width: number): void => {
    this.width = clampPanel(width);
  };

  persist = (): void => {
    writeStored(KEY, { width: this.width });
  };
}

export const mapPanel = new MapPanel();
