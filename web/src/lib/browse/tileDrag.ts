import type { Point } from '#lib/gestures/point.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { getContext, setContext } from 'svelte';

export interface TouchPlace {
  escapes(point: Point): boolean;
  begin(ids: readonly string[], point: Point): void;
  move(point: Point): void;
  drop(point: Point): void;
  cancel(): void;
}

export interface TileDrag {
  start(asset: AssetSummary, event: DragEvent): void;
  end(): void;
  readonly touch?: TouchPlace;
}

const KEY = Symbol('tileDrag');

export function provideTileDrag(drag: TileDrag): void {
  setContext(KEY, drag);
}

export function tileDrag(): TileDrag | null {
  return getContext<TileDrag | undefined>(KEY) ?? null;
}
