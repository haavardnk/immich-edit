import { createHost } from '#lib/commands/host.ts';
import type { Point } from '#lib/gestures/point.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export interface GridHost {
  items: () => readonly AssetSummary[];
  selectingAll: () => boolean;
  selectAll: () => Promise<boolean>;
  cancelSelectAll: () => void;
  move: (key: string, extend: boolean) => void;
  reveal: (id: string) => void;
  scrollTo: (id: string) => void;
  focusBulkBar: () => void;
  holdAnchor: (at: Point | null) => () => void;
}

export const gridHost = createHost<GridHost>();
