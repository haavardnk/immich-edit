import {
  ImagePreloader,
  neighbourWindow,
  preloadUrls,
  type Neighbour,
  type StepFn
} from '#lib/browse/loupePreload.ts';
import { describe, expect, it, vi } from 'vitest';

function strip(ids: string[]): StepFn {
  return (id, direction) => {
    const next = ids[ids.indexOf(id) + direction];
    return next === undefined ? null : { id: next };
  };
}

function ids(neighbours: Neighbour[]): string[] {
  return neighbours.map(({ asset }) => asset.id);
}

const IDS = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];

function render(id: string): string {
  return `render:${id}`;
}

describe('neighbourWindow', () => {
  it('walks both ways, nearest first, next before previous', () => {
    expect(neighbourWindow('e', strip(IDS), 3)).toEqual([
      { asset: { id: 'f' }, distance: 1 },
      { asset: { id: 'd' }, distance: 1 },
      { asset: { id: 'g' }, distance: 2 },
      { asset: { id: 'c' }, distance: 2 },
      { asset: { id: 'h' }, distance: 3 },
      { asset: { id: 'b' }, distance: 3 }
    ]);
  });

  it('stops at the loaded ends', () => {
    expect(ids(neighbourWindow('b', strip(IDS), 3))).toEqual(['c', 'a', 'd', 'e']);
    expect(neighbourWindow('a', strip(['a']), 3)).toEqual([]);
  });

  it('follows the step function across hidden assets', () => {
    const visible = ['a', 'c', 'e', 'g'];
    expect(ids(neighbourWindow('c', strip(visible), 2))).toEqual(['e', 'a', 'g']);
  });
});

describe('preloadUrls', () => {
  it('preloads Immich previews three deep and renders one deep', () => {
    expect(preloadUrls({ id: 'e' }, strip(IDS), render)).toEqual([
      '/api/assets/f/thumb?size=preview',
      'render:f',
      '/api/assets/d/thumb?size=preview',
      'render:d',
      '/api/assets/g/thumb?size=preview',
      '/api/assets/c/thumb?size=preview',
      '/api/assets/h/thumb?size=preview',
      '/api/assets/b/thumb?size=preview'
    ]);
  });

  it('shares one Immich preview between a master and its copies', () => {
    const master = '0b7c3c1e-5f2a-4c8e-9d1b-2a3f4e5d6c7b';
    expect(preloadUrls({ id: master }, strip([master, `${master}_1`, 'b']), render)).toEqual([
      `render:${master}_1`,
      '/api/assets/b/thumb?size=preview'
    ]);
  });

  it('versions each Immich preview with its thumbhash and checksum', () => {
    const versioned: StepFn = (id, direction) =>
      id === 'a' && direction === 1 ? { id: 'b', thumbhash: 'hb', checksum: 'sb' } : null;
    expect(preloadUrls({ id: 'a', thumbhash: 'ha', checksum: 'sa' }, versioned, render)).toEqual([
      '/api/assets/b/thumb?size=preview&c=5f3ae86f3551ffa0',
      'render:b'
    ]);
  });
});

describe('ImagePreloader', () => {
  it('loads each wanted image once and lets go of the ones that left', () => {
    const load = vi.fn((src: string) => ({ src }) as HTMLImageElement);
    const preloader = new ImagePreloader(load);

    preloader.keep(['x', 'y']);
    preloader.keep(['y', 'z']);
    expect(load.mock.calls.map(([src]) => src)).toEqual(['x', 'y', 'z']);

    preloader.keep(['x']);
    expect(load.mock.calls.map(([src]) => src)).toEqual(['x', 'y', 'z', 'x']);

    preloader.keep([]);
    preloader.keep(['x']);
    expect(load).toHaveBeenCalledTimes(5);
  });

  it('aborts the download of an image that left the window', () => {
    const images = new Map<string, { src: string }>();
    const preloader = new ImagePreloader((src) => {
      const image = { src };
      images.set(src, image);
      return image as HTMLImageElement;
    });

    preloader.keep(['x', 'y']);
    preloader.keep(['y']);

    expect(images.get('x')?.src).toBe('');
    expect(images.get('y')?.src).toBe('y');
  });
});
