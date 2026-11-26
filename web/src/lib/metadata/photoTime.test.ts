import {
  cameraTime,
  immichZone,
  offsetLabel,
  wallToInstant,
  zoneLabel,
  zoneOffsetMinutes
} from '#lib/metadata/photoTime.ts';
import { asset } from '#lib/testing/assets.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';

describe('camera time', () => {
  it('knows zone offsets across daylight saving', () => {
    expect(zoneOffsetMinutes('Europe/Oslo', Date.parse('2026-07-01T12:00:00Z'))).toBe(120);
    expect(zoneOffsetMinutes('Europe/Oslo', Date.parse('2026-12-01T12:00:00Z'))).toBe(60);
    expect(zoneOffsetMinutes('Asia/Kolkata', Date.parse('2026-12-01T12:00:00Z'))).toBe(330);
    expect(zoneOffsetMinutes('UTC', 0)).toBe(0);
  });

  it('turns wall time in a zone into an instant', () => {
    expect(wallToInstant(Date.parse('2026-10-31T10:42:05Z'), 'Europe/Oslo')).toBe(
      Date.parse('2026-10-31T09:42:05Z')
    );
    expect(wallToInstant(Date.parse('2026-10-24T10:42:05Z'), 'Europe/Oslo')).toBe(
      Date.parse('2026-10-24T08:42:05Z')
    );
  });

  it('trusts a recorded zone and reads wall time in the camera zone otherwise', () => {
    const phone = asset('p', {
      localDateTime: '2026-10-31T10:42:05.000Z',
      exifInfo: {
        dateTimeOriginal: '2026-10-31T09:42:05.000Z',
        timeZone: 'Europe/Oslo'
      } as AssetSummary['exifInfo']
    });
    expect(cameraTime(phone, 'Asia/Tokyo')).toEqual({
      instant: Date.parse('2026-10-31T09:42:05Z'),
      recorded: true
    });
    const camera = asset('c', {
      localDateTime: '2026-10-31T10:42:05.000Z',
      exifInfo: {
        dateTimeOriginal: '2026-10-31T10:42:05.000Z',
        timeZone: null
      } as AssetSummary['exifInfo']
    });
    expect(cameraTime(camera, 'Europe/Oslo')).toEqual({
      instant: Date.parse('2026-10-31T09:42:05Z'),
      recorded: false
    });
    expect(cameraTime(asset('x'), 'UTC')).toBeNull();
  });

  it('labels offsets', () => {
    expect(offsetLabel(60)).toBe('UTC+1');
    expect(offsetLabel(-210)).toBe('UTC−3:30');
    expect(offsetLabel(0)).toBe('UTC+0');
  });

  it('names a fixed offset once and a region with its offset', () => {
    const summer = Date.parse('2026-07-01T12:00:00Z');
    expect(zoneLabel('UTC+1', summer)).toBe('UTC+1');
    expect(zoneLabel('UTC-4', summer)).toBe('UTC−4');
    expect(zoneLabel('UTC+5:30', summer)).toBe('UTC+5:30');
    expect(zoneLabel('America/New_York', summer)).toBe('America/New York · UTC−4');
  });

  it('names zones the way Immich stores them', () => {
    expect(immichZone('Europe/Oslo', Date.parse('2026-07-01T12:00:00Z'))).toBe('UTC+2');
    expect(immichZone('Asia/Kolkata', Date.parse('2026-07-01T12:00:00Z'))).toBe('UTC+5:30');
    expect(immichZone('America/St_Johns', Date.parse('2026-01-01T12:00:00Z'))).toBe('UTC-3:30');
    expect(immichZone('UTC', 0)).toBe('UTC+0');
  });
});
