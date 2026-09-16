import type { ScopeGrid } from '$lib/types/preview';

export type Raster = {
  width: number;
  height: number;
  data: Uint8ClampedArray<ArrayBuffer>;
};

const LUMA_TINT: [number, number, number] = [0.9, 1, 0.92];
const VECTOR_LUMA = 0.6;

function scale(value: number, gain: number): number {
  return Math.min(255, value * gain);
}

export function rasterizeLuma(grid: ScopeGrid, gain: number): Raster {
  const data = new Uint8ClampedArray(grid.width * grid.height * 4);
  for (let i = 0; i < grid.width * grid.height; i++) {
    const v = scale(grid.data[i] ?? 0, gain);
    data[i * 4] = v * LUMA_TINT[0];
    data[i * 4 + 1] = v * LUMA_TINT[1];
    data[i * 4 + 2] = v * LUMA_TINT[2];
    data[i * 4 + 3] = 255;
  }
  return { width: grid.width, height: grid.height, data };
}

export function rasterizeOverlay(grid: ScopeGrid, gain: number): Raster {
  const data = new Uint8ClampedArray(grid.width * grid.height * 4);
  for (let i = 0; i < grid.width * grid.height; i++) {
    data[i * 4] = scale(grid.data[i * 3] ?? 0, gain);
    data[i * 4 + 1] = scale(grid.data[i * 3 + 1] ?? 0, gain);
    data[i * 4 + 2] = scale(grid.data[i * 3 + 2] ?? 0, gain);
    data[i * 4 + 3] = 255;
  }
  return { width: grid.width, height: grid.height, data };
}

export function rasterizeParade(grid: ScopeGrid, gain: number): Raster {
  const width = grid.width * 3;
  const data = new Uint8ClampedArray(width * grid.height * 4);
  for (let y = 0; y < grid.height; y++) {
    for (let x = 0; x < width; x++) {
      const channel = Math.floor(x / grid.width);
      const source = (y * grid.width + (x % grid.width)) * 3 + channel;
      const target = (y * width + x) * 4;
      data[target + channel] = scale(grid.data[source] ?? 0, gain);
      data[target + 3] = 255;
    }
  }
  return { width, height: grid.height, data };
}

export function rasterizeVector(grid: ScopeGrid, gain: number): Raster {
  const size = grid.width;
  const data = new Uint8ClampedArray(size * grid.height * 4);
  for (let y = 0; y < grid.height; y++) {
    for (let x = 0; x < size; x++) {
      const i = y * size + x;
      const v = scale(grid.data[i] ?? 0, gain);
      if (v === 0) {
        data[i * 4 + 3] = 255;
        continue;
      }
      const [r, g, b] = cellHue(x, y, size);
      data[i * 4] = v * r;
      data[i * 4 + 1] = v * g;
      data[i * 4 + 2] = v * b;
      data[i * 4 + 3] = 255;
    }
  }
  return { width: size, height: grid.height, data };
}

function cellHue(x: number, y: number, size: number): [number, number, number] {
  const cb = (x + 0.5) / size - 0.5;
  const cr = 0.5 - (y + 0.5) / size;
  const r = VECTOR_LUMA + 1.5748 * cr;
  const b = VECTOR_LUMA + 1.8556 * cb;
  const g = (VECTOR_LUMA - 0.2126 * r - 0.0722 * b) / 0.7152;
  const peak = Math.max(r, g, b, 0.001);
  return [Math.max(0, r) / peak, Math.max(0, g) / peak, Math.max(0, b) / peak];
}

export function rasterize(grid: ScopeGrid, gain: number, parade: boolean): Raster {
  if (grid.kind === 'vectorscope') return rasterizeVector(grid, gain);
  if (grid.kind === 'waveform') return rasterizeLuma(grid, gain);
  return parade ? rasterizeParade(grid, gain) : rasterizeOverlay(grid, gain);
}
