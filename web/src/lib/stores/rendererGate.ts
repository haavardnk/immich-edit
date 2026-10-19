import { formatBytes, perMegapixel } from '$lib/diagnostics/format';
import type { GpuLimits, TickSample } from '$lib/render/protocol';

export const SPEED_MARGIN = 0.8;
const AVERAGE_WEIGHT = 0.3;
const LINEAR_TEXEL_BYTES = 8;

export type RendererUse = 'browser' | 'server';

export interface Verdict {
  use: RendererUse | null;
  reason: string;
}

export interface GateInput {
  browserMs: number | null;
  serverMs: number | null;
  limits: GpuLimits | null;
  edge: number;
}

export function msPerMegapixel({ ms, pixels }: TickSample): number | null {
  if (!(pixels > 0) || !Number.isFinite(ms) || ms < 0) return null;
  return (ms * 1_000_000) / pixels;
}

export function blend(average: number | null, sample: number): number {
  return average === null ? sample : average + (sample - average) * AVERAGE_WEIGHT;
}

export function limitsShortfall(limits: GpuLimits, edge: number): string | null {
  if (limits.max_texture_dimension_2d < edge) {
    return `GPU textures max ${limits.max_texture_dimension_2d} px, need ${edge} px`;
  }
  const bytes = edge * edge * LINEAR_TEXEL_BYTES;
  if (limits.max_buffer_size < bytes) {
    return `GPU buffers max ${formatBytes(limits.max_buffer_size)}, need ${formatBytes(bytes)}`;
  }
  return null;
}

export function decide({ browserMs, serverMs, limits, edge }: GateInput): Verdict {
  const shortfall = limits && limitsShortfall(limits, edge);
  if (shortfall) return { use: 'server', reason: shortfall };
  if (browserMs === null || serverMs === null) {
    return { use: null, reason: 'measuring speed' };
  }
  const speeds = `${browserMs.toFixed(1)} vs ${perMegapixel(serverMs)}`;
  if (browserMs <= serverMs * SPEED_MARGIN) {
    return { use: 'browser', reason: `browser faster, ${speeds}` };
  }
  return { use: 'server', reason: `browser not clearly faster, ${speeds}` };
}
