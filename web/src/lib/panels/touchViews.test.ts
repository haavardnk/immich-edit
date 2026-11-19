import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
import { describe, expect, it } from 'vitest';
import { TOUCH_VIEW_PARTS, touchViewKey } from './touchViews';

describe('touch view parts', () => {
  it('renders exactly the catalog touch views', () => {
    const catalog = TOUCH_GROUPS.flatMap((group) =>
      group.views.map((view) => touchViewKey(group.id, view.id))
    );
    expect(Object.keys(TOUCH_VIEW_PARTS).sort()).toEqual([...catalog].sort());
  });

  it('gives every view at least one part', () => {
    for (const parts of Object.values(TOUCH_VIEW_PARTS)) expect(parts.length).toBeGreaterThan(0);
  });
});
