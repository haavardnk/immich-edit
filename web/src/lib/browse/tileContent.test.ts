import { album } from '#lib/testing/albums.ts';
import { asset } from '#lib/testing/assets.ts';
import type { ExifInfo, TagRef } from '#lib/types/asset.ts';
import { labelTagValue } from '#lib/stores/labels.ts';
import { describe, expect, it } from 'vitest';
import { clusterItems, tileDetails, topMarks, type TileFacts } from './tileContent';
import { OVERLAY_DETAILS, OVERLAY_ICONS, type TileOverlay } from './tileOverlay';

const ID = '0f8fad5b-d9cb-469f-a165-70867728950e';

const EVERYTHING: TileOverlay = { icons: OVERLAY_ICONS, details: OVERLAY_DETAILS };

const NO_FACTS: TileFacts = { edited: false, albums: null, tags: null };

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

function tag(id: string, value: string): TagRef {
  return { id, name: value.split('/').at(-1) ?? value, value };
}

const reject = tag('r', 'immich-edit/reject');
const red = tag('l', labelTagValue('red'));

describe('top marks', () => {
  it('shows the chosen culling marks the photo carries', () => {
    const photo = asset(ID, { isFavorite: true, tags: [reject, red] });
    expect(topMarks(EVERYTHING, photo, { ...NO_FACTS, edited: true })).toEqual([
      'edited',
      'favorite',
      'rejected',
      'label'
    ]);
    expect(topMarks({ icons: ['label'], details: [] }, photo, NO_FACTS)).toEqual(['label']);
    expect(topMarks(EVERYTHING, asset(ID), NO_FACTS)).toEqual([]);
  });
});

describe('cluster items', () => {
  it('orders metadata icons before copy and rating, counting user tags only', () => {
    const photo = asset(`${ID}_2`, {
      originalFileName: 'DSC_0042.ARW',
      copyLabel: 'B&W',
      tags: [reject],
      exifInfo: exif({ latitude: 60.39, longitude: 5.32, description: ' Fjord ', rating: 4 })
    });
    const facts: TileFacts = {
      edited: false,
      albums: [album('a1'), album('a2')],
      tags: [reject, tag('t1', 'Trips/Norway'), tag('t2', 'Family')]
    };
    expect(clusterItems(EVERYTHING, photo, facts)).toEqual([
      { key: 'location' },
      { key: 'tags', count: 2 },
      { key: 'description' },
      { key: 'albums', count: 2 },
      { key: 'format', text: 'ARW' },
      { key: 'copy', text: 'B&W' },
      { key: 'rating', rating: 4 }
    ]);
  });

  it('skips what the photo lacks and what is not chosen', () => {
    const photo = asset(ID, {
      originalFileName: 'noext',
      exifInfo: exif({ description: '   ', latitude: 200, longitude: 5 })
    });
    expect(clusterItems(EVERYTHING, photo, { ...NO_FACTS, tags: [reject], albums: [] })).toEqual(
      []
    );
    expect(clusterItems({ icons: ['format'], details: [] }, asset(`${ID}_3`), NO_FACTS)).toEqual([
      { key: 'format', text: 'JPG' }
    ]);
  });

  it('names an unlabelled copy by its index', () => {
    expect(clusterItems({ icons: ['copy'], details: [] }, asset(`${ID}_3`), NO_FACTS)).toEqual([
      { key: 'copy', text: 'Copy 3' }
    ]);
  });
});

describe('tile details', () => {
  const photo = asset(ID, {
    originalFileName: 'IMG_1.CR3',
    exifInfo: exif({
      city: 'Bergen',
      country: 'Norway',
      description: 'Line one\nline two',
      focalLength: 35,
      fNumber: 2.8,
      exposureTime: '0.004',
      iso: 400,
      make: 'Canon',
      model: 'EOS R5',
      lensModel: 'RF 35mm F1.8',
      exifImageWidth: 6000,
      exifImageHeight: 4000,
      fileSizeInByte: 25480396
    })
  });
  const facts: TileFacts = {
    edited: false,
    albums: [album('a', { albumName: 'Autumn' }), album('b', { albumName: 'Hikes' })],
    tags: [reject, tag('t', 'Trips/Norway')]
  };

  it('formats every detail', () => {
    expect(tileDetails(EVERYTHING, photo, facts, 10).map((line) => [line.key, line.text])).toEqual([
      ['filename', 'IMG_1.CR3'],
      ['place', 'Bergen, Norway'],
      ['tags', 'Norway'],
      ['description', 'Line one line two'],
      ['albums', 'Autumn, Hikes'],
      ['exposure', '35mm · f/2.8 · 1/250s · ISO 400'],
      ['camera', 'Canon EOS R5'],
      ['lens', 'RF 35mm F1.8'],
      ['size', '6000 × 4000 · 24.3 MB']
    ]);
  });

  it('keeps the first lines that fit, skipping empty ones', () => {
    const overlay: TileOverlay = { icons: [], details: ['filename', 'taken', 'albums', 'lens'] };
    expect(tileDetails(overlay, photo, NO_FACTS, 2).map((line) => line.key)).toEqual([
      'filename',
      'lens'
    ]);
    expect(tileDetails(overlay, photo, NO_FACTS, 0)).toEqual([]);
  });
});
