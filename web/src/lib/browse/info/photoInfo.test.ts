import {
  detailRows,
  exposureCells,
  fileType,
  mapUrl,
  placeOf,
  takenLabel
} from '#lib/browse/info/photoInfo.ts';
import type { ExifInfo } from '#lib/types/asset.ts';
import { describe, expect, it } from 'vitest';

function exif(fields: Partial<ExifInfo>): ExifInfo {
  return {
    make: null,
    model: null,
    lensModel: null,
    city: null,
    state: null,
    country: null,
    fNumber: null,
    focalLength: null,
    iso: null,
    exposureTime: null,
    exifImageWidth: null,
    exifImageHeight: null,
    orientation: null,
    dateTimeOriginal: null,
    timeZone: null,
    rating: null,
    fileSizeInByte: null,
    description: null,
    latitude: null,
    longitude: null,
    ...fields
  };
}

const full = exif({
  make: 'SONY',
  model: 'ILCE-7M4',
  lensModel: 'FE 35mm F1.4 GM',
  fNumber: 2.8,
  exposureTime: '0.004',
  iso: 400,
  focalLength: 35,
  exifImageWidth: 7008,
  exifImageHeight: 4672,
  fileSizeInByte: 48.2 * 1024 * 1024
});

describe('exposure strip', () => {
  it('lists aperture, shutter, ISO and focal length in that order', () => {
    expect(exposureCells({ exifInfo: full })).toEqual([
      { label: 'Aperture', value: 'f/2.8' },
      { label: 'Shutter', value: '1/250s' },
      { label: 'ISO', value: '400' },
      { label: 'Focal', value: '35mm' }
    ]);
  });

  it('marks a missing value and drops the strip when nothing is known', () => {
    expect(exposureCells({ exifInfo: exif({ iso: 100 }) }).map((cell) => cell.value)).toEqual([
      '—',
      '—',
      '100',
      '—'
    ]);
    expect(exposureCells({ exifInfo: exif({ make: 'SONY' }) })).toEqual([]);
    expect(exposureCells({ exifInfo: null })).toEqual([]);
  });
});

describe('detail rows', () => {
  it('keeps the editor rows without the exposure and adds the file type to the size', () => {
    const rows = detailRows({ originalFileName: 'IMG_4120.arw', exifInfo: full });
    expect(rows).toEqual([
      { key: 'Camera', value: 'SONY ILCE-7M4' },
      { key: 'Lens', value: 'FE 35mm F1.4 GM' },
      { key: 'Dimensions', value: '7008 × 4672' },
      { key: 'Size', value: '48.2 MB · ARW' }
    ]);
  });

  it('reads the type from the last dot only', () => {
    expect(fileType('a.b.dng')).toBe('DNG');
    expect(fileType('README')).toBeNull();
    expect(fileType('.hidden')).toBeNull();
    expect(fileType('trailing.')).toBeNull();
  });
});

describe('place', () => {
  it('names the city and lists the region under it', () => {
    expect(
      placeOf(
        exif({
          city: 'Bergen',
          state: 'Vestland',
          country: 'Norway',
          latitude: 60.39,
          longitude: 5.32
        })
      )
    ).toEqual({
      name: 'Bergen',
      region: 'Vestland, Norway',
      mapUrl: mapUrl(60.39, 5.32)
    });
  });

  it('falls back to the coordinates and has no map link without them', () => {
    expect(placeOf(exif({ latitude: -33.8568, longitude: 151.2153 }))?.name).toBe(
      '33.8568° S, 151.2153° E'
    );
    expect(placeOf(exif({ country: 'Norway' }))).toEqual({
      name: 'Norway',
      region: null,
      mapUrl: null
    });
    expect(placeOf(exif({ latitude: 95, longitude: 10 }))).toBeNull();
    expect(placeOf(exif({}))).toBeNull();
    expect(placeOf(null)).toBeNull();
  });

  it('links to OpenStreetMap at the photo', () => {
    expect(mapUrl(60.391263, 5.322054)).toBe(
      'https://www.openstreetmap.org/?mlat=60.39126&mlon=5.32205#map=14/60.39126/5.32205'
    );
  });
});

describe('taken label', () => {
  it('reads the local capture time with weekday and minutes', () => {
    const label = takenLabel({ localDateTime: '2026-07-14T19:42:07.000Z', exifInfo: null });
    const at = new Date('2026-07-14T19:42:07.000Z');
    const day = at.toLocaleDateString(undefined, {
      timeZone: 'UTC',
      weekday: 'short',
      day: 'numeric',
      month: 'short',
      year: 'numeric'
    });
    const time = at.toLocaleTimeString(undefined, {
      timeZone: 'UTC',
      hour: '2-digit',
      minute: '2-digit'
    });
    expect(label).toBe(`${day} · ${time}`);
    expect(takenLabel({ exifInfo: null })).toBeNull();
  });
});
