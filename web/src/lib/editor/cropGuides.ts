import type { CropGrid } from '#lib/stores/ui.svelte.ts';

export type GuidePoint = readonly [number, number];
export type GuidePath = readonly GuidePoint[];

export const GUIDE_TURNS = 8;
export const TURNING_GRIDS: ReadonlySet<CropGrid> = new Set(['spiral', 'triangle']);

const PHI = (1 + Math.sqrt(5)) / 2;
const GOLDEN = 1 - 1 / PHI;
const SPIRAL_ARCS = 9;
const ARC_POINTS = 24;

function axisLines(stops: readonly number[]): GuidePath[] {
  return stops.flatMap((t): GuidePath[] => [
    [
      [t, 0],
      [t, 1]
    ],
    [
      [0, t],
      [1, t]
    ]
  ]);
}

function footOnDiagonal(px: number, py: number, w: number, h: number): GuidePoint {
  const t = (px * w + py * h) / (w * w + h * h);
  return [t, t];
}

function triangle(aspect: number): GuidePath[] {
  const w = Math.max(aspect, 1e-6);
  const h = 1;
  return [
    [
      [0, 0],
      [1, 1]
    ],
    [[1, 0], footOnDiagonal(w, 0, w, h)],
    [[0, 1], footOnDiagonal(0, h, w, h)]
  ];
}

type Square = { x: number; y: number; size: number };

const SPIRAL_CORNERS: readonly (readonly [number, number])[] = [
  [1, 1],
  [0, 1],
  [0, 0],
  [1, 0]
];

function spiralSquares(): Square[] {
  const squares: Square[] = [];
  let x = 0;
  let y = 0;
  let w = PHI;
  let h = 1;
  for (let i = 0; i < SPIRAL_ARCS; i++) {
    const side = Math.min(w, h);
    const step = i % 4;
    if (step === 0) {
      squares.push({ x, y, size: side });
      x += side;
      w -= side;
    } else if (step === 1) {
      squares.push({ x, y, size: side });
      y += side;
      h -= side;
    } else if (step === 2) {
      squares.push({ x: x + w - side, y, size: side });
      w -= side;
    } else {
      squares.push({ x, y: y + h - side, size: side });
      h -= side;
    }
  }
  return squares;
}

function spiral(): GuidePath[] {
  const points = spiralSquares().flatMap((square, i) => {
    const corner = SPIRAL_CORNERS[i % 4] ?? [1, 1];
    const cx = square.x + corner[0] * square.size;
    const cy = square.y + corner[1] * square.size;
    const start = Math.PI + (i % 4) * (Math.PI / 2);
    return Array.from({ length: ARC_POINTS + 1 }, (_, k): GuidePoint => {
      const angle = start + (k / ARC_POINTS) * (Math.PI / 2);
      return [(cx + Math.cos(angle) * square.size) / PHI, cy + Math.sin(angle) * square.size];
    });
  });
  return [points];
}

function baseGuides(grid: CropGrid, aspect: number): GuidePath[] {
  switch (grid) {
    case 'thirds':
      return axisLines([1 / 3, 2 / 3]);
    case 'golden':
      return axisLines([GOLDEN, 1 - GOLDEN]);
    case 'spiral':
      return spiral();
    case 'diagonal':
      return [
        [
          [0, 0],
          [1, 1]
        ],
        [
          [1, 0],
          [0, 1]
        ]
      ];
    case 'triangle':
      return triangle(aspect);
    case 'grid':
      return axisLines([1, 2, 3, 4, 5, 6, 7].map((i) => i / 8));
    case 'off':
      return [];
  }
}

export function turnPoint([x, y]: GuidePoint, turn: number): GuidePoint {
  const step = ((turn % GUIDE_TURNS) + GUIDE_TURNS) % GUIDE_TURNS;
  const swapped: GuidePoint = step >= 4 ? [y, x] : [x, y];
  const flipX = step % 4 === 1 || step % 4 === 2;
  const flipY = step % 4 === 2 || step % 4 === 3;
  return [flipX ? 1 - swapped[0] : swapped[0], flipY ? 1 - swapped[1] : swapped[1]];
}

export function cropGuides(grid: CropGrid, turn: number, aspect: number): GuidePath[] {
  const transposed = ((turn % GUIDE_TURNS) + GUIDE_TURNS) % GUIDE_TURNS >= 4;
  const base = baseGuides(grid, transposed ? 1 / Math.max(aspect, 1e-6) : aspect);
  if (!TURNING_GRIDS.has(grid)) return base;
  return base.map((path) => path.map((point) => turnPoint(point, turn)));
}

export function guidePoints(path: GuidePath): string {
  return path.map(([x, y]) => `${x},${y}`).join(' ');
}
