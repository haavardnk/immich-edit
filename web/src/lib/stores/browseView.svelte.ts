import { compare } from './compare.svelte';
import { readStored, writeStored } from '#lib/utils/storage.ts';
import { isPanelSectionId, type PanelSectionId } from '#lib/browse/filterSections.ts';
import {
  dayHeaderHeight,
  defaultGridSize,
  gridRowHeight,
  isGridSize,
  type GridSize
} from '#lib/browse/gridSizes.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import type { TileAnchor } from '#lib/browse/gridZoom.ts';

export type TileInfo = 'none' | 'badges' | 'full';

const TILE_INFO_ORDER: TileInfo[] = ['none', 'badges', 'full'];

const STORAGE_KEY = 'immich-edit:browseView';

type Persisted = {
  filtersOpen: boolean;
  filterSections: PanelSectionId[];
  gridSizeChoice?: GridSize;
  loupeAutoAdvance: boolean;
  loupeInfoOpen: boolean;
  tileInfo: TileInfo;
};

class BrowseViewStore {
  #chosenSize = $state<GridSize | null>(null);
  activeId = $state<string | null>(null);
  loupeId = $state<string | null>(null);
  loupeInfoOpen = $state(false);
  loupeTagsOpen = $state(false);
  loupeAutoAdvance = $state(false);
  tileInfo = $state<TileInfo>('badges');
  #filtersDocked = $state(false);
  #filtersSheet = $state(false);
  #filtersCompact = $state(false);
  filterSections = $state<PanelSectionId[]>([]);
  editorReturnLoupeId = $state<string | null>(null);
  lastGridPath = $state<string | null>(null);
  private gridAnchors = new Map<string, TileAnchor>();

  constructor() {
    const stored = readStored<Persisted>(STORAGE_KEY);
    const size = stored?.gridSizeChoice;
    if (isGridSize(size)) this.#chosenSize = size;
    if (typeof stored?.loupeAutoAdvance === 'boolean')
      this.loupeAutoAdvance = stored.loupeAutoAdvance;
    if (typeof stored?.loupeInfoOpen === 'boolean') this.loupeInfoOpen = stored.loupeInfoOpen;
    if (stored?.tileInfo && TILE_INFO_ORDER.includes(stored.tileInfo))
      this.tileInfo = stored.tileInfo;
    if (typeof stored?.filtersOpen === 'boolean') this.#filtersDocked = stored.filtersOpen;
    if (Array.isArray(stored?.filterSections))
      this.filterSections = stored.filterSections.filter(isPanelSectionId);
  }

  private persist(): void {
    writeStored(STORAGE_KEY, {
      filtersOpen: this.#filtersDocked,
      filterSections: this.filterSections,
      gridSizeChoice: this.#chosenSize ?? undefined,
      loupeAutoAdvance: this.loupeAutoAdvance,
      loupeInfoOpen: this.loupeInfoOpen,
      tileInfo: this.tileInfo
    } satisfies Persisted);
  }

  get gridSize(): GridSize {
    return this.#chosenSize ?? defaultGridSize(layout);
  }

  get rowHeight(): number {
    return gridRowHeight(this.gridSize, layout.mode);
  }

  get dayHeaderHeight(): number {
    return dayHeaderHeight(layout.mode);
  }

  setGridSize(size: GridSize): void {
    this.#chosenSize = size;
    this.persist();
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
    if (next) this.setTileInfo(next);
  }

  setTileInfo(info: TileInfo): void {
    this.tileInfo = info;
    this.persist();
  }

  get filtersOpen(): boolean {
    return this.#filtersCompact ? this.#filtersSheet : this.#filtersDocked;
  }

  setFiltersOpen(open: boolean): void {
    if (this.#filtersCompact) {
      this.#filtersSheet = open;
      return;
    }
    this.#filtersDocked = open;
    this.persist();
  }

  setFiltersCompact(compact: boolean): void {
    if (compact === this.#filtersCompact) return;
    this.#filtersCompact = compact;
    this.#filtersSheet = false;
  }

  setFilterSection(id: PanelSectionId, open: boolean): void {
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

  setGridAnchor(key: string, anchor: TileAnchor | null): void {
    if (!key) return;
    if (anchor) this.gridAnchors.set(key, anchor);
    else this.gridAnchors.delete(key);
  }

  getGridAnchor(key: string): TileAnchor | null {
    return this.gridAnchors.get(key) ?? null;
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
