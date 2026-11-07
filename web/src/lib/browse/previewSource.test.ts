import { paneNatural, paneSources, previewStatusLabel } from '#lib/browse/previewSource.ts';
import { describe, expect, it } from 'vitest';

const MASTER = '0b7c3c1e-5f2a-4c8e-9d1b-2a3f4e5d6c7b';

describe('paneSources', () => {
  it('lays the Immich preview under a render that has not loaded', () => {
    expect(paneSources('a', null, 1536, false)).toEqual({
      render: '/api/assets/a/preview?max=1536&clip=false',
      underlay: '/api/assets/a/thumb?size=preview'
    });
  });

  it('drops the underlay once the render for the asset has loaded', () => {
    expect(paneSources('a', 'a', 2048, true)).toEqual({
      render: '/api/assets/a/preview?max=2048&clip=true',
      underlay: null
    });
  });

  it('keeps the underlay while only another asset has rendered', () => {
    expect(paneSources('b', 'a', 1024, false).underlay).toBe('/api/assets/b/thumb?size=preview');
  });

  it('asks Immich for the master preview of a virtual copy', () => {
    expect(paneSources(`${MASTER}_2`, null, 1024, false)).toEqual({
      render: `/api/assets/${MASTER}_2/preview?max=1024&clip=false`,
      underlay: `/api/assets/${MASTER}/thumb?size=preview`
    });
  });
});

describe('paneNatural', () => {
  const render = { id: 'a', w: 2048, h: 1365 };
  const preview = { id: 'b', w: 1440, h: 960 };

  it('prefers the render of the shown asset', () => {
    expect(paneNatural('a', render, { ...preview, id: 'a' })).toEqual({ w: 2048, h: 1365 });
  });

  it('measures by the Immich preview while the render is pending', () => {
    expect(paneNatural('b', render, preview)).toEqual({ w: 1440, h: 960 });
  });

  it('keeps no size until an image of the new asset loads', () => {
    expect(paneNatural('c', render, preview)).toEqual({ w: 0, h: 0 });
    expect(paneNatural('c', null, preview)).toEqual({ w: 0, h: 0 });
  });

  it('is empty before anything loads', () => {
    expect(paneNatural('a', null, null)).toEqual({ w: 0, h: 0 });
  });
});

describe('previewStatusLabel', () => {
  it('names the edits when the photo has them', () => {
    expect(previewStatusLabel(true)).toBe('Immich preview · rendering edits');
    expect(previewStatusLabel(false)).toBe('Immich preview · rendering');
  });
});
