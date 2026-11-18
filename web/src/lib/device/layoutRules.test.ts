import { resolveLayout, type Layout, type LayoutInput } from '#lib/device/layoutRules.ts';
import { describe, expect, it } from 'vitest';

type Row = [string, LayoutInput, Layout];

const ROWS: Row[] = [
  [
    'phone portrait',
    { width: 390, height: 844, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: false, landscape: false, compact: true }
  ],
  [
    'phone landscape',
    { width: 844, height: 390, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: false, landscape: true, compact: true }
  ],
  [
    'iPad portrait',
    { width: 820, height: 1180, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: false, landscape: false, compact: true }
  ],
  [
    'iPad landscape',
    { width: 1180, height: 820, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: true, landscape: true, compact: false }
  ],
  [
    'iPad mini landscape',
    { width: 1133, height: 744, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: true, landscape: true, compact: false }
  ],
  [
    'small touch tablet landscape',
    { width: 760, height: 620, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: false, landscape: true, compact: true }
  ],
  [
    'touch tablet landscape narrower than wide',
    { width: 960, height: 600, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: false, landscape: true, compact: true }
  ],
  [
    'iPad Pro landscape',
    { width: 1366, height: 1024, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: true, landscape: true, compact: false }
  ],
  [
    'small laptop window',
    { width: 700, height: 800, coarse: false, choice: 'auto' },
    { mode: 'touch', wide: false, landscape: false, compact: true }
  ],
  [
    'laptop',
    { width: 1280, height: 800, coarse: false, choice: 'auto' },
    { mode: 'desktop', wide: false, landscape: true, compact: false }
  ],
  [
    'full HD touch screen',
    { width: 1920, height: 1080, coarse: true, choice: 'auto' },
    { mode: 'touch', wide: true, landscape: true, compact: false }
  ],
  [
    'large touch monitor',
    { width: 2560, height: 1440, coarse: true, choice: 'auto' },
    { mode: 'desktop', wide: false, landscape: true, compact: false }
  ],
  [
    'full HD desktop',
    { width: 1920, height: 1080, coarse: false, choice: 'auto' },
    { mode: 'desktop', wide: false, landscape: true, compact: false }
  ],
  [
    'desktop forced on a phone',
    { width: 390, height: 844, coarse: true, choice: 'desktop' },
    { mode: 'desktop', wide: false, landscape: false, compact: false }
  ],
  [
    'desktop forced on an iPad landscape',
    { width: 1180, height: 820, coarse: true, choice: 'desktop' },
    { mode: 'desktop', wide: false, landscape: true, compact: false }
  ],
  [
    'touch forced on a laptop',
    { width: 1280, height: 800, coarse: false, choice: 'touch' },
    { mode: 'touch', wide: true, landscape: true, compact: false }
  ],
  [
    'touch forced on a large monitor',
    { width: 1920, height: 1080, coarse: false, choice: 'touch' },
    { mode: 'touch', wide: true, landscape: true, compact: false }
  ]
];

describe('resolveLayout', () => {
  it.each(ROWS)('%s', (_name, input, expected) => {
    expect(resolveLayout(input)).toEqual(expected);
  });
});
