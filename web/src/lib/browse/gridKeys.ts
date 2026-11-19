import { verticalAssetIndex, type AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import type { WindowEnd } from '#lib/browse/feedWindow.ts';

export type GridMove =
  | { kind: 'index'; index: number }
  | { kind: 'edge'; end: WindowEnd }
  | { kind: 'beyond'; end: WindowEnd };

export type MoveContext = {
  layout: AssetGridLayout;
  count: number;
  cursor: number;
  selecting: boolean;
  pageRows: number;
  hasPrevious: boolean;
  hasMore: boolean;
};

const VERTICAL: Readonly<Record<string, -1 | 1>> = {
  ArrowUp: -1,
  ArrowDown: 1,
  PageUp: -1,
  PageDown: 1
};

function clamped(index: number, ctx: MoveContext): GridMove {
  if (index < 0) return ctx.hasPrevious ? { kind: 'beyond', end: 'first' } : at(0);
  if (index >= ctx.count) {
    return ctx.hasMore ? { kind: 'beyond', end: 'last' } : at(ctx.count - 1);
  }
  return at(index);
}

function at(index: number): GridMove {
  return { kind: 'index', index: Math.max(0, index) };
}

export function pageRows(viewHeight: number, rowHeight: number): number {
  return Math.max(1, Math.floor(viewHeight / Math.max(1, rowHeight)));
}

export function moveIndex(key: string, ctx: MoveContext): GridMove {
  if (key === 'Home') return ctx.hasPrevious ? { kind: 'edge', end: 'first' } : at(0);
  if (key === 'End') return ctx.hasMore ? { kind: 'edge', end: 'last' } : at(ctx.count - 1);
  if (ctx.cursor < 0 || !ctx.selecting) return at(ctx.cursor);
  if (key === 'ArrowRight') return clamped(ctx.cursor + 1, ctx);
  if (key === 'ArrowLeft') return clamped(ctx.cursor - 1, ctx);
  const direction = VERTICAL[key];
  if (!direction) return at(ctx.cursor);
  const box = ctx.layout.boxes[ctx.cursor];
  if (!box) return at(ctx.cursor);
  const rows = key.startsWith('Page') ? ctx.pageRows : 1;
  const row = box.row + direction * rows;
  if (row < 0 && ctx.hasPrevious) return { kind: 'beyond', end: 'first' };
  if (row >= ctx.layout.rows.length && ctx.hasMore) return { kind: 'beyond', end: 'last' };
  return at(verticalAssetIndex(ctx.layout, ctx.cursor, direction * rows));
}
