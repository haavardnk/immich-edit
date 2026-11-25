import { describe, expect, it } from 'vitest';
import { planShifts, shiftCaption, shiftText, wholeMinutes } from './captureShift';

const OSLO_WINTER = Date.UTC(2026, 0, 15, 10);
const OSLO_SUMMER = Date.UTC(2026, 6, 15, 10);

describe('capture time shifts', () => {
  it('rounds a correction to whole minutes', () => {
    expect(wholeMinutes(-(62 * 60 + 14) * 1000)).toBe(-62);
    expect(wholeMinutes(-(4 * 3600 + 59 * 60 + 38) * 1000)).toBe(-300);
    expect(wholeMinutes(20_000)).toBe(0);
  });

  it('shifts photos with a zone by the correction and undoes it', () => {
    const plan = planShifts(
      [{ id: 'a', instant: OSLO_WINTER, recorded: true }],
      -62 * 60_000,
      'Europe/Oslo'
    );
    expect(plan.write).toEqual([{ id: 'a', minutes: -62 }]);
    expect(plan.undo).toEqual([{ id: 'a', minutes: 62 }]);
  });

  it('skips photos with a zone when the correction rounds to nothing', () => {
    const plan = planShifts(
      [{ id: 'a', instant: OSLO_WINTER, recorded: true }],
      20_000,
      'Europe/Oslo'
    );
    expect(plan.write).toEqual([]);
    expect(plan.undo).toEqual([]);
  });

  it('gives a photo without a zone the zone and keeps its wall time, per DST offset', () => {
    const plan = planShifts(
      [
        { id: 'w', instant: OSLO_WINTER, recorded: false },
        { id: 's', instant: OSLO_SUMMER, recorded: false }
      ],
      -62 * 60_000,
      'Europe/Oslo'
    );
    expect(plan.write).toEqual([
      { id: 'w', minutes: -122, timeZone: 'Europe/Oslo' },
      { id: 's', minutes: -182, timeZone: 'Europe/Oslo' }
    ]);
    expect(plan.undo).toEqual([
      { id: 'w', minutes: 62 },
      { id: 's', minutes: 62 }
    ]);
    expect(plan.zoned).toBe(2);
  });

  it('still writes the zone when there is no correction, and has nothing to undo', () => {
    const plan = planShifts([{ id: 'w', instant: OSLO_WINTER, recorded: false }], 0, 'Europe/Oslo');
    expect(plan.write).toEqual([{ id: 'w', minutes: -60, timeZone: 'Europe/Oslo' }]);
    expect(plan.undo).toEqual([]);
  });

  it('names the shift simply', () => {
    expect(shiftText(-62)).toBe('1 h 02 min earlier');
    expect(shiftText(5)).toBe('5 min later');
    expect(shiftText(-300)).toBe('5 h earlier');
    const plan = planShifts(
      [{ id: 'w', instant: OSLO_WINTER, recorded: false }],
      -62 * 60_000,
      'America/New_York'
    );
    expect(shiftCaption(plan, 'America/New_York')).toBe(
      '1 h 02 min earlier · America/New York for 1 without a zone'
    );
  });
});
