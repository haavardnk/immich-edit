import { describe, expect, it } from 'vitest';
import { rasterizeLuma, rasterizeOverlay, rasterizeParade, rasterizeVector } from './rasterize';
import type { ScopeGrid, ScopeKind } from '$lib/types/preview';

function grid(kind: ScopeKind, width: number, height: number, channels: number): ScopeGrid {
  const data = new Uint8Array(width * height * channels);
  for (let i = 0; i < data.length; i++) data[i] = (i * 7) % 256;
  return { kind, width, height, channels, maxCount: 1000, data };
}

describe('rasterize', () => {
  it('tints luma density and keeps the grid shape', () => {
    const raster = rasterizeLuma(grid('waveform', 4, 2, 1), 1);
    expect(raster.width).toBe(4);
    expect(raster.height).toBe(2);
    expect(raster.data.length).toBe(4 * 2 * 4);
    expect(raster.data[3]).toBe(255);
  });

  it.each([
    [1, 40],
    [4, 160],
    [8, 255]
  ])('applies %ix gain with clamping', (gain, expected) => {
    const source: ScopeGrid = {
      kind: 'waveform',
      width: 1,
      height: 1,
      channels: 1,
      maxCount: 10,
      data: new Uint8Array([40])
    };
    expect(rasterizeLuma(source, gain).data[1]).toBe(expected);
  });

  it('maps parade channels to three side by side bands', () => {
    const source: ScopeGrid = {
      kind: 'parade',
      width: 2,
      height: 1,
      channels: 3,
      maxCount: 9,
      data: new Uint8Array([10, 20, 30, 40, 50, 60])
    };
    const raster = rasterizeParade(source, 1);
    expect(raster.width).toBe(6);
    expect([...raster.data.slice(0, 4)]).toEqual([10, 0, 0, 255]);
    expect([...raster.data.slice(8, 12)]).toEqual([0, 20, 0, 255]);
    expect([...raster.data.slice(20, 24)]).toEqual([0, 0, 60, 255]);
  });

  it('overlays parade channels on a shared axis', () => {
    const source: ScopeGrid = {
      kind: 'parade',
      width: 1,
      height: 1,
      channels: 3,
      maxCount: 9,
      data: new Uint8Array([10, 20, 30])
    };
    expect([...rasterizeOverlay(source, 1).data]).toEqual([10, 20, 30, 255]);
  });

  it('colours vectorscope cells by their hue position', () => {
    const source: ScopeGrid = {
      kind: 'vectorscope',
      width: 8,
      height: 8,
      channels: 1,
      maxCount: 100,
      data: new Uint8Array(64).fill(200)
    };
    const raster = rasterizeVector(source, 1);
    const left = (4 * 8 + 0) * 4;
    const right = (4 * 8 + 7) * 4;
    expect(raster.data[right + 2]).toBeGreaterThan(raster.data[left + 2] ?? 0);
    expect(raster.data[left + 1]).toBeGreaterThan(raster.data[right + 1] ?? 0);
  });

  it('leaves empty vectorscope cells opaque black', () => {
    const source: ScopeGrid = {
      kind: 'vectorscope',
      width: 2,
      height: 1,
      channels: 1,
      maxCount: 1,
      data: new Uint8Array([0, 0])
    };
    expect([...rasterizeVector(source, 4).data]).toEqual([0, 0, 0, 255, 0, 0, 0, 255]);
  });
});
