import { describe, expect, it } from 'vitest';
import { camerasText, changeText } from './cameraChanges';

const HOUR = 3_600_000;

describe('camera changes', () => {
  it('names the time zone and the shift', () => {
    expect(changeText('Africa/Dar_es_Salaam', 0)).toBe('Dar es Salaam');
    expect(changeText(null, -HOUR)).toBe('−1:00:00');
    expect(changeText('Asia/Kolkata', 22_000)).toBe('Kolkata · +0:00:22');
    expect(changeText(null, 400)).toBe('');
  });

  it('sums up the cameras that changed', () => {
    expect(camerasText([{ label: 'ILCE-7M3', change: '' }])).toBe('No change');
    expect(camerasText([{ label: 'ILCE-7M3', change: '−1:00:00' }])).toBe('−1:00:00');
    expect(
      camerasText([
        { label: 'ILCE-7M3', change: 'Dar es Salaam' },
        { label: 'ILCE-7M4', change: '' },
        { label: 'iPhone 15 Pro', change: '' }
      ])
    ).toBe('ILCE-7M3 Dar es Salaam · 2 unchanged');
  });
});
