import { describe, expect, it } from 'vitest';

const sources = import.meta.glob<string>(['/src/**/*.svelte', '/src/**/*.ts'], {
  query: '?raw',
  import: 'default',
  eager: true
});

const UTILITY = '(?:bg|text|border|ring|from|via|to|outline|divide|fill|stroke|placeholder|caret)';

const LITERALS = [
  new RegExp(`\\b${UTILITY}-(?:neutral|zinc|slate|stone)-\\d+(?:/\\d+)?`, 'g'),
  new RegExp(`\\b${UTILITY}-white(?:/\\d+)?(?![\\w-])`, 'g'),
  new RegExp(`\\b${UTILITY}-black(?![\\w/-])`, 'g'),
  /#[0-9a-fA-F]{3,8}\b/g,
  /rgba?\(/g
];

const IMAGE_SURFACES: Record<string, string[]> = {
  '/src/lib/components/browse/AssetTile.svelte': [
    'border-white',
    'text-white',
    'text-white/70',
    'text-white/75'
  ],
  '/src/lib/components/browse/Loupe.svelte': ['bg-neutral-950', 'text-white'],
  '/src/lib/components/browse/LoupePane.svelte': [
    'bg-neutral-950/85',
    'border-white/20',
    'border-white/35',
    'text-neutral-950',
    'text-white/90'
  ],
  '/src/lib/components/browse/LoupePaneGrid.svelte': ['text-white'],
  '/src/lib/components/editor/BrushCanvas.svelte': ['border-white/90', 'rgb(', 'rgba('],
  '/src/lib/components/editor/ClickCanvas.svelte': ['border-white', 'ring-white/60', 'text-white'],
  '/src/lib/components/editor/CropOverlay.svelte': ['bg-white', 'border-white/90', 'rgb('],
  '/src/lib/components/editor/UprightGuides.svelte': ['border-white/40', 'rgb(', 'text-white'],
  '/src/lib/components/editor/Viewer.svelte': [
    'bg-white',
    'bg-white/90',
    'ring-white/10',
    'text-white',
    'text-white/85'
  ],
  '/src/lib/components/editor/WhiteBalancePicker.svelte': ['text-white', 'text-white/60'],
  '/src/lib/components/preset/PresetTile.svelte': ['text-white/90'],
  '/src/lib/components/shell/Filmstrip.svelte': ['text-white'],
  '/src/lib/panels/MaskRangeControls.svelte': ['rgb(']
};

function isScanned(path: string): boolean {
  if (path.includes('.test.') || path.includes('/lib/wasm/')) return false;
  return !path.startsWith('/src/lib/theme/');
}

function literalsIn(source: string): string[] {
  return LITERALS.flatMap((pattern) => [...source.matchAll(pattern)].map((match) => match[0]));
}

describe('chrome colours', () => {
  const found = Object.fromEntries(
    Object.entries(sources)
      .filter(([path]) => isScanned(path))
      .map(([path, source]) => [path, [...new Set(literalsIn(source))].sort()] as const)
      .filter(([, literals]) => literals.length > 0)
  );

  it('use theme tokens outside image surfaces', () => {
    expect(found).toEqual(IMAGE_SURFACES);
  });
});
