import { createHost } from '#lib/commands/host.ts';

export interface LoupeHost {
  cycleZoom: () => void;
  panBy: (dx: number, dy: number) => void;
}

export const loupeHost = createHost<LoupeHost>();
