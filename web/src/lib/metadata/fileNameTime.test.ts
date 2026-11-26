import { nameTime, withTimeOfDay } from '#lib/metadata/fileNameTime.ts';
import { describe, expect, it } from 'vitest';

const iso = (name: string): string | null => {
  const found = nameTime(name);
  if (!found) return null;
  const flags = `${found.dateOnly ? ' day' : ''}${found.utc ? ' utc' : ''}`;
  return `${found.source} ${new Date(found.wall).toISOString()}${flags}`;
};

describe('capture time from file names', () => {
  it('reads the common phone and app patterns', () => {
    expect(iso('IMG-20231105-WA0001.jpg')).toBe('WhatsApp 2023-11-05T00:00:00.000Z day');
    expect(iso('VID-20231105-WA0012.mp4')).toBe('WhatsApp 2023-11-05T00:00:00.000Z day');
    expect(iso('PXL_20240712_181522347.jpg')).toBe('Pixel 2024-07-12T18:15:22.347Z utc');
    expect(iso('signal-2024-03-02-101500.jpg')).toBe('Signal 2024-03-02T10:15:00.000Z');
    expect(iso('signal-2024-03-02-10-15-00-123.jpg')).toBe('Signal 2024-03-02T10:15:00.123Z');
    expect(iso('Screenshot_20240102-093015.png')).toBe('Screenshot 2024-01-02T09:30:15.000Z');
    expect(iso('Screenshot 2024-01-02 at 09.30.15.png')).toBe(
      'Screenshot 2024-01-02T09:30:15.000Z'
    );
    expect(iso('20231105_123456.jpg')).toBe('Date and time 2023-11-05T12:34:56.000Z');
    expect(iso('IMG_20231105_123456.jpg')).toBe('Date and time 2023-11-05T12:34:56.000Z');
    expect(iso('2023-11-05 12.34.56.jpg')).toBe('Date and time 2023-11-05T12:34:56.000Z');
  });

  it('ignores names without a real date', () => {
    expect(nameTime('IMG_0042.JPG')).toBeNull();
    expect(nameTime('DSC04112.ARW')).toBeNull();
    expect(nameTime('holiday-scan.png')).toBeNull();
    expect(nameTime('20231340_123456.jpg')).toBeNull();
    expect(nameTime('IMG-20230230-WA0001.jpg')).toBeNull();
    expect(nameTime('18990101_120000.jpg')).toBeNull();
  });

  it('puts a date-only name at the time of day the photo shows now', () => {
    const day = Date.parse('2023-11-05T00:00:00Z');
    expect(new Date(withTimeOfDay(day, Date.parse('2026-11-26T08:14:02Z'))).toISOString()).toBe(
      '2023-11-05T08:14:02.000Z'
    );
  });
});
