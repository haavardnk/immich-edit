import { compare } from './compare.svelte';
import { readStored, writeStored } from '$lib/utils/storage';
import { isFilterSectionId, type FilterSectionId } from '$lib/browse/filterSections';

export type GridSize = 'sm' | 'md' | 'lg' | 'xl';
export type TileInfo = 'none' | 'badges' | 'full';

const TILE_INFO_ORDER: TileInfo[] = ['none', 'badges', 'full'];

const GRID_MIN: Record<GridSize, number> = {
  sm: 110,
  md: 150,
  lg: 220,
  xl: 320
};

const SIZE_ORDER: GridSize[] = ['sm', 'md', 'lg', 'xl'];

const STORAGE_KEY = 'immich-edit:browseView';

type Persisted = {
  filtersOpen: boolean;
  filterSections: FilterSectionId[];
  gridSize: GridSize;
  loupeAutoAdvance: boolean;
  loupeInfoOpen: boolean;
  tileInfo: TileInfo;
};

class BrowseViewStore {
  gridSize = $state<GridSize>('md');
  activeId = $state<string | null>(null);
  loupeId = $state<string | null>(null);
  loupeInfoOpen = $state(false);
  loupeTagsOpen = $state(false);
  loupeAutoAdvance = $state(false);
  tileInfo = $state<TileInfo>('badges');
  filtersOpen = $state(false);
  filterSections = $state<FilterSectionId[]>([]);
  editorReturnLoupeId = $state<string | null>(null);
  lastGridPath = $state<string | null>(null);
  private gridScroll = new Map<string, number>();

  constructor() {
    const stored = readStored<Persisted>(STORAGE_KEY);
    if (stored?.gridSize && SIZE_ORDER.includes(stored.gridSize)) this.gridSize = stored.gridSize;
    if (typeof stored?.loupeAutoAdvance === 'boolean')
      this.loupeAutoAdvance = stored.loupeAutoAdvance;
    if (typeof stored?.loupeInfoOpen === 'boolean') this.loupeInfoOpen = stored.loupeInfoOpen;
    if (stored?.tileInfo && TILE_INFO_ORDER.includes(stored.tileInfo))
      this.tileInfo = stored.tileInfo;
    if (typeof stored?.filtersOpen === 'boolean') this.filtersOpen = stored.filtersOpen;
    if (Array.isArray(stored?.filterSections))
      this.filterSections = stored.filterSections.filter(isFilterSectionId);
  }

  private persist(): void {
    writeStored(STORAGE_KEY, {
      filtersOpen: this.filtersOpen,
      filterSections: this.filterSections,
      gridSize: this.gridSize,
      loupeAutoAdvance: this.loupeAutoAdvance,
      loupeInfoOpen: this.loupeInfoOpen,
      tileInfo: this.tileInfo
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
    const size = SIZE_ORDER[next];
    if (size) this.setGridSize(size);
  }

  setLoupeAutoAdvance(on: boolean): void {
    this.loupeAutoAdvance = on;
    this.persist();
  }

  toggleLoupeInfo(): void {
    this.loupeInfoOpen = !this.loupeInfoOpen;
    this.persist();
  }

  cycleTileInfo(): void {
    const next =
      TILE_INFO_ORDER[(TILE_INFO_ORDER.indexOf(this.tileInfo) + 1) % TILE_INFO_ORDER.length];
    if (next) this.tileInfo = next;
    this.persist();
  }

  setFiltersOpen(open: boolean): void {
    this.filtersOpen = open;
    this.persist();
  }

  setFilterSection(id: FilterSectionId, open: boolean): void {
    const rest = this.filterSections.filter((section) => section !== id);
    this.filterSections = open ? [...rest, id] : rest;
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
