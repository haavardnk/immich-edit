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
  new RegExp(`(?<!dark:)\\b${UTILITY}-black/\\d+`, 'g'),
  /#[0-9a-fA-F]{3,8}\b/g,
  /rgba?\(/g
];

const IMAGE_SURFACES: Record<string, string[]> = {
  '/src/lib/components/browse/AssetTile.svelte': [
    'bg-black/35',
    'bg-black/40',
    'bg-black/55',
    'bg-black/70',
    'border-white',
    'from-black/90',
    'text-white',
    'text-white/70',
    'text-white/75',
    'via-black/55'
  ],
  '/src/lib/components/browse/Loupe.svelte': ['bg-neutral-950'],
  '/src/lib/components/browse/LoupePane.svelte': [
    'bg-neutral-950/85',
    'border-white/20',
    'border-white/35',
    'text-neutral-950',
    'text-white/90'
  ],
  '/src/lib/components/browse/LoupePaneGrid.svelte': ['bg-black/50', 'bg-black/75', 'text-white'],
  '/src/lib/components/browse/LoupeStepButtons.svelte': [
    'bg-black/40',
    'bg-black/70',
    'text-white'
  ],
  '/src/lib/components/browse/PeekView.svelte': [
    'bg-black/90',
    'text-white',
    'text-white/40',
    'text-white/85'
  ],
  '/src/lib/components/browse/PreviewStatusChip.svelte': ['bg-black/70', 'text-white/85'],
  '/src/lib/components/browse/RatingHud.svelte': [
    'bg-black/70',
    'bg-white/15',
    'text-white',
    'text-white/45',
    'text-white/70'
  ],
  '/src/lib/components/editor/BrushCanvas.svelte': ['border-white/90', 'rgb(', 'rgba('],
  '/src/lib/components/editor/ClickCanvas.svelte': [
    'bg-black/70',
    'border-white',
    'ring-white/60',
    'text-white'
  ],
  '/src/lib/components/editor/CropOverlay.svelte': ['border-white/90'],
  '/src/lib/components/editor/crop/CropGuide.svelte': ['rgb('],
  '/src/lib/components/editor/crop/GuideLoupe.svelte': [
    'bg-black',
    'bg-white',
    'ring-black/60',
    'ring-white'
  ],
  '/src/lib/components/editor/crop/CropHandles.svelte': ['bg-white', 'border-white'],
  '/src/lib/components/editor/FloatingScope.svelte': [
    'bg-black/45',
    'bg-black/60',
    'bg-white/35',
    'bg-white/95',
    'ring-white/14',
    'text-white',
    'text-white/90'
  ],
  '/src/lib/components/editor/Navigator.svelte': ['border-white', 'rgb(', 'ring-white/14'],
  '/src/lib/components/editor/PerspectiveCorners.svelte': ['border-black/60'],
  '/src/lib/components/editor/PhotoPill.svelte': ['bg-black/70', 'text-white', 'text-white/70'],
  '/src/lib/components/editor/TargetCallout.svelte': [
    'bg-black/75',
    'text-white',
    'text-white/60',
    'text-white/80'
  ],
  '/src/lib/components/editor/UprightGuides.svelte': [
    'bg-black/70',
    'bg-white',
    'border-black/60',
    'border-white/40',
    'rgb(',
    'text-white'
  ],
  '/src/lib/components/editor/Viewer.svelte': ['bg-black/70', 'ring-white/10', 'text-white/85'],
  '/src/lib/components/editor/WhiteBalancePicker.svelte': [
    'bg-black/75',
    'text-white',
    'text-white/60'
  ],
  '/src/lib/components/editor/compare/CompareLabel.svelte': ['bg-black/55', 'text-white'],
  '/src/lib/components/editor/compare/CompareSplit.svelte': [
    'bg-white',
    'bg-white/90',
    'border-black/40',
    'text-black/70'
  ],
  '/src/lib/components/editor/versions/VersionThumb.svelte': ['bg-black/60', 'text-white'],
  '/src/lib/components/preset/PresetTile.svelte': ['bg-black/55', 'from-black/80', 'text-white/90'],
  '/src/lib/components/shell/Filmstrip.svelte': ['from-black/75', 'text-white'],
  '/src/lib/components/viewer/ViewerSplit.svelte': ['bg-white', 'bg-white/90'],
  '/src/lib/components/viewer/ViewerStatus.svelte': ['bg-black/70', 'text-white'],
  '/src/lib/components/viewer/ViewerWindow.svelte': ['ring-white/10'],
  '/src/lib/panels/MaskRangeControls.svelte': ['rgb(']
};

const SCRIMS: Record<string, string[]> = {
  '/src/lib/components/jobs/JobsDrawer.svelte': ['bg-black/60'],
  '/src/lib/components/sheet/SheetScrim.svelte': ['bg-black/20', 'bg-black/60']
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

  it('use theme tokens outside image surfaces and scrims', () => {
    expect(found).toEqual({ ...IMAGE_SURFACES, ...SCRIMS });
  });
});
