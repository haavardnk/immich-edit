import { browsing } from './browsing.svelte';

export type GridSize = 'sm' | 'md' | 'lg' | 'xl';

const GRID_MIN: Record<GridSize, number> = {
  sm: 110,
  md: 150,
  lg: 220,
  xl: 320
};

const SIZE_ORDER: GridSize[] = ['sm', 'md', 'lg', 'xl'];

const STORAGE_KEY = 'immich-edit:browseView';

type Persisted = {
  gridSize: GridSize;
  loupeAutoAdvance: boolean;
};

function loadPersisted(): Persisted {
  const fallback: Persisted = { gridSize: 'md', loupeAutoAdvance: false };
  if (typeof localStorage === 'undefined') return fallback;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    return {
      gridSize: SIZE_ORDER.includes(parsed.gridSize as GridSize)
        ? (parsed.gridSize as GridSize)
        : fallback.gridSize,
      loupeAutoAdvance:
        typeof parsed.loupeAutoAdvance === 'boolean'
          ? parsed.loupeAutoAdvance
          : fallback.loupeAutoAdvance
    };
  } catch {
    return fallback;
  }
}

class BrowseViewStore {
  gridSize = $state<GridSize>('md');
  activeId = $state<string | null>(null);
  loupeId = $state<string | null>(null);
  loupeZoomed = $state(false);
  loupePanX = $state(0);
  loupePanY = $state(0);
  loupeInfoOpen = $state(false);
  loupeTagsOpen = $state(false);
  loupeAutoAdvance = $state(false);

  constructor() {
    const p = loadPersisted();
    this.gridSize = p.gridSize;
    this.loupeAutoAdvance = p.loupeAutoAdvance;
  }

  private persist(): void {
    if (typeof localStorage === 'undefined') return;
    const data: Persisted = {
      gridSize: this.gridSize,
      loupeAutoAdvance: this.loupeAutoAdvance
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  }

  get minTile(): number {
    return GRID_MIN[this.gridSize];
  }

  setGridSize(size: GridSize): void {
    this.gridSize = size;
    this.persist();
  }

  stepGridSize(delta: number): void {
    const idx = SIZE_ORDER.indexOf(this.gridSize);
    const next = Math.min(SIZE_ORDER.length - 1, Math.max(0, idx + delta));
    this.setGridSize(SIZE_ORDER[next]);
  }

  setLoupeAutoAdvance(on: boolean): void {
    this.loupeAutoAdvance = on;
    this.persist();
  }

  setActive(id: string | null): void {
    this.activeId = id;
  }

  moveActive(delta: number): string | null {
    const list = browsing.assets;
    if (list.length === 0) return null;
    const idx = this.activeId ? list.findIndex((a) => a.id === this.activeId) : -1;
    if (idx < 0) {
      this.activeId = list[0].id;
      return this.activeId;
    }
    const next = Math.min(list.length - 1, Math.max(0, idx + delta));
    this.activeId = list[next].id;
    return this.activeId;
  }

  resetLoupeView(): void {
    this.loupeZoomed = false;
    this.loupePanX = 0;
    this.loupePanY = 0;
  }

  openLoupe(id: string): void {
    this.activeId = id;
    this.loupeId = id;
    this.resetLoupeView();
    this.loupeTagsOpen = false;
  }

  closeLoupe(): void {
    if (this.loupeId) this.activeId = this.loupeId;
    this.loupeId = null;
    this.resetLoupeView();
    this.loupeTagsOpen = false;
  }
}

export const browseView = new BrowseViewStore();
