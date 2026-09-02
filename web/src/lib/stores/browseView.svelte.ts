import { browsing } from './browsing.svelte';
import { compare } from './compare.svelte';
import { readStored, writeStored } from '$lib/utils/storage';

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

class BrowseViewStore {
  gridSize = $state<GridSize>('md');
  activeId = $state<string | null>(null);
  loupeId = $state<string | null>(null);
  loupeInfoOpen = $state(false);
  loupeTagsOpen = $state(false);
  loupeAutoAdvance = $state(false);
  editorReturnLoupeId = $state<string | null>(null);
  lastGridPath = $state<string | null>(null);
  private gridScroll = new Map<string, number>();

  constructor() {
    const stored = readStored<Persisted>(STORAGE_KEY);
    if (stored?.gridSize && SIZE_ORDER.includes(stored.gridSize)) this.gridSize = stored.gridSize;
    if (typeof stored?.loupeAutoAdvance === 'boolean')
      this.loupeAutoAdvance = stored.loupeAutoAdvance;
  }

  private persist(): void {
    writeStored(STORAGE_KEY, {
      gridSize: this.gridSize,
      loupeAutoAdvance: this.loupeAutoAdvance
    } satisfies Persisted);
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

  setLastGridPath(path: string): void {
    this.lastGridPath = path;
  }

  setGridScroll(key: string, top: number): void {
    if (!key) return;
    this.gridScroll.set(key, Math.max(0, top));
  }

  getGridScroll(key: string): number {
    if (!key) return 0;
    return this.gridScroll.get(key) ?? 0;
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
    compare.exit();
  }

  openLoupe(id: string): void {
    this.activeId = id;
    this.loupeId = id;
    this.resetLoupeView();
    this.loupeTagsOpen = false;
  }

  leaveLoupeForEditor(id: string): void {
    this.editorReturnLoupeId = id;
    this.closeLoupe();
  }

  consumeEditorReturnLoupe(id: string | null): boolean {
    const returnToLoupe = id !== null && this.editorReturnLoupeId === id;
    this.editorReturnLoupeId = null;
    return returnToLoupe;
  }

  closeLoupe(): void {
    if (this.loupeId) this.activeId = this.loupeId;
    this.loupeId = null;
    this.resetLoupeView();
    this.loupeTagsOpen = false;
  }
}

export const browseView = new BrowseViewStore();
