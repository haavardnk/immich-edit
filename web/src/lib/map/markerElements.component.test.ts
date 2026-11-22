import { clusterElement, markedOf, showMarked } from '#lib/map/markerElements.ts';
import { describe, expect, it } from 'vitest';

describe('marker elements', () => {
  it('reports how much of a marker is selected', () => {
    const selected = (id: string) => id === 'a' || id === 'b';
    expect(markedOf(['a', 'b'], selected)).toBe('all');
    expect(markedOf(['a', 'c'], selected)).toBe('some');
    expect(markedOf(['c'], selected)).toBe('none');
  });

  it('adds a check only while every photo is selected', () => {
    const element = clusterElement(2);
    showMarked(element, 'all');
    expect(element.dataset['marked']).toBe('all');
    expect(element.querySelectorAll('[data-mark-check]')).toHaveLength(1);
    showMarked(element, 'some');
    expect(element.querySelector('[data-mark-check]')).toBeNull();
  });
});
