import { createHost } from '#lib/commands/host.ts';

export interface FilterPanelHost {
  close: () => Promise<void>;
}

export const filterPanelHost = createHost<FilterPanelHost>();
