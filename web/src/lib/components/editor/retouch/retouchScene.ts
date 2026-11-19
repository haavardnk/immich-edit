import type { RetouchStroke, Vec2f } from '#lib/types/edits.ts';
import {
  displayUvToSceneUv,
  sceneUvToDisplayUv,
  type ViewTransform
} from '#lib/utils/canvasCoords.ts';
import { crosshair, link, paintPath, ring, type Px } from './retouchDraw';

export type RetouchColours = { active: string; source: string; light: string };

export type RetouchScene = {
  view: ViewTransform;
  strokes: readonly RetouchStroke[];
  activeId: string | null;
  anchor: Vec2f | null;
  toolSize: number;
  drawPts: readonly Px[];
  strokeOffset: Px | null;
  hover: Px | null;
  picking: boolean;
  ghost: boolean;
  colours: RetouchColours;
  radius: (sceneRadius: number, nx: number, ny: number) => number;
};

function toPx(view: ViewTransform, point: Vec2f, w: number, h: number): Px {
  const [x, y] = sceneUvToDisplayUv(view, point.x, point.y);
  return [x * w, y * h];
}

function drawStrokes(
  ctx: CanvasRenderingContext2D,
  layer: HTMLCanvasElement,
  scene: RetouchScene
): void {
  const { width: w, height: h } = ctx.canvas;
  const { colours } = scene;
  for (const stroke of scene.strokes) {
    if (!stroke.enabled) continue;
    const pts = stroke.points.map((p) => toPx(scene.view, p, w, h));
    const head = pts[0];
    if (!head) continue;
    const active = stroke.id === scene.activeId;
    const rPx = scene.radius(stroke.radius, head[0] / w, head[1] / h);
    const tint = active ? colours.active : colours.light;
    paintPath(ctx, layer, pts, rPx, {
      fill: tint,
      fillAlpha: active ? 0.2 : 0.05,
      line: tint,
      lineAlpha: active ? 0.9 : 0.45
    });
    if (!active) continue;
    const source = toPx(scene.view, stroke.source, w, h);
    link(ctx, head, source, colours.active, 0.5);
    ring(ctx, source[0], source[1], rPx, colours.source, true, 0.95);
  }
}

function drawLive(
  ctx: CanvasRenderingContext2D,
  layer: HTMLCanvasElement,
  scene: RetouchScene
): void {
  const { width: w, height: h } = ctx.canvas;
  const first = scene.drawPts[0];
  const last = scene.drawPts[scene.drawPts.length - 1];
  if (!first || !last) return;
  const pts = scene.drawPts.map(([x, y]) => [x * w, y * h] as Px);
  const rPx = scene.radius(scene.toolSize, first[0], first[1]);
  const tint = scene.colours.active;
  paintPath(ctx, layer, pts, rPx, { fill: tint, fillAlpha: 0.25, line: tint, lineAlpha: 0.9 });
  if (!scene.strokeOffset) return;
  const [sx, sy] = displayUvToSceneUv(scene.view, last[0], last[1]);
  const source = toPx(
    scene.view,
    { x: sx + scene.strokeOffset[0], y: sy + scene.strokeOffset[1] },
    w,
    h
  );
  ring(ctx, source[0], source[1], rPx, scene.colours.source, true, 0.95);
  crosshair(ctx, source[0], source[1], scene.colours.source, 0.95);
}

function drawCursor(ctx: CanvasRenderingContext2D, scene: RetouchScene, hover: Px): void {
  const { width: w, height: h } = ctx.canvas;
  const { colours } = scene;
  const at: Px = [hover[0] * w, hover[1] * h];
  const rPx = scene.radius(scene.toolSize, hover[0], hover[1]);
  if (scene.ghost && scene.anchor) {
    link(ctx, at, toPx(scene.view, scene.anchor, w, h), colours.source, 0.7);
  }
  const tint = scene.picking ? colours.source : colours.light;
  ring(ctx, at[0], at[1], rPx, tint, scene.picking, scene.picking ? 0.9 : 0.8);
  if (scene.picking) crosshair(ctx, at[0], at[1], colours.source, 0.9);
}

export function drawRetouch(
  ctx: CanvasRenderingContext2D,
  layer: HTMLCanvasElement,
  scene: RetouchScene
): void {
  const { width: w, height: h } = ctx.canvas;
  ctx.clearRect(0, 0, w, h);
  drawStrokes(ctx, layer, scene);
  const drawing = scene.drawPts.length > 0;
  if (scene.anchor && !drawing) {
    const [ax, ay] = toPx(scene.view, scene.anchor, w, h);
    const rPx = scene.radius(scene.toolSize, ax / w, ay / h);
    ring(ctx, ax, ay, rPx, scene.colours.source, true, 0.95);
    crosshair(ctx, ax, ay, scene.colours.source, 0.95);
  }
  if (drawing) {
    drawLive(ctx, layer, scene);
    return;
  }
  if (scene.hover) drawCursor(ctx, scene, scene.hover);
}
