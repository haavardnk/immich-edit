import { createHost } from '#lib/commands/host.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export interface GridHost {
  items: () => readonly AssetSummary[];
  selectingAll: () => boolean;
  selectAll: () => Promise<boolean>;
  move: (key: string, extend: boolean) => void;
  reveal: (id: string) => void;
  focusBulkBar: () => void;
}

export const gridHost = createHost<GridHost>();
