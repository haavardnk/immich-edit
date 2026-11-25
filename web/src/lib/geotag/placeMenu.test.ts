import { describe, expect, it } from 'vitest';
import { pickerPlaceActions, pickerPlaceMenu } from '#lib/geotag/placeMenu.ts';

describe('picker place menu', () => {
  it('offers moving a place to the pin only when the pin is elsewhere', () => {
    expect(pickerPlaceMenu(true)).toEqual([['pin', 'rename'], ['remove']]);
    expect(pickerPlaceMenu(false)).toEqual([['rename'], ['remove']]);
    expect(pickerPlaceActions('Hytta').pin.label).toBe('Move Hytta to the pin');
  });
});
