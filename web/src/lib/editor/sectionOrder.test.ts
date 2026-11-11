import { describe, expect, it } from 'vitest';
import {
  moveWithin,
  orderSections,
  reorderSections,
  storedSections,
  toggleSection
} from './sectionOrder';

const DEFAULTS = ['a', 'b', 'c', 'd'];

describe('section order', () => {
  it('starts in default order with nothing hidden', () => {
    expect(orderSections(DEFAULTS, null).map((entry) => entry.id)).toEqual(DEFAULTS);
  });

  it('follows the stored order, drops unknown ids and slots in new ones after their neighbour', () => {
    const entries = orderSections(DEFAULTS, [
      { id: 'c', hidden: true },
      { id: 'z', hidden: false },
      { id: 'a', hidden: false },
      { id: 'c', hidden: false }
    ]);
    expect(entries).toEqual([
      { id: 'c', hidden: true },
      { id: 'd', hidden: false },
      { id: 'a', hidden: false },
      { id: 'b', hidden: false }
    ]);
  });

  it('stores null for the default layout', () => {
    expect(storedSections(orderSections(DEFAULTS, null), DEFAULTS)).toBeNull();
    const hidden = toggleSection(orderSections(DEFAULTS, null), 'b');
    expect(storedSections(hidden, DEFAULTS)).toEqual([
      { id: 'a', hidden: false },
      { id: 'b', hidden: true },
      { id: 'c', hidden: false },
      { id: 'd', hidden: false }
    ]);
    expect(storedSections(toggleSection(hidden, 'b'), DEFAULTS)).toBeNull();
  });

  it('reorders a subset in place', () => {
    const entries = reorderSections(orderSections(DEFAULTS, null), ['c', 'b']);
    expect(entries.map((entry) => entry.id)).toEqual(['a', 'c', 'b', 'd']);
  });

  it('moves one id within an order', () => {
    expect(moveWithin(['a', 'b', 'c'], 'a', 2)).toEqual(['b', 'c', 'a']);
    expect(moveWithin(['a', 'b', 'c'], 'c', -1)).toEqual(['c', 'a', 'b']);
  });
});
