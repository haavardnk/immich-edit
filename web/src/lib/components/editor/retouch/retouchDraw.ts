export type Px = [number, number];

export function ring(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  r: number,
  colour: string,
  dashed: boolean,
  alpha: number
): void {
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.setLineDash(dashed ? [4, 3] : []);
  ctx.strokeStyle = colour;
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.arc(x, y, Math.max(1, r), 0, Math.PI * 2);
  ctx.stroke();
  ctx.restore();
}

export function crosshair(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  colour: string,
  alpha: number
): void {
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.strokeStyle = colour;
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(x - 5, y);
  ctx.lineTo(x + 5, y);
  ctx.moveTo(x, y - 5);
  ctx.lineTo(x, y + 5);
  ctx.stroke();
  ctx.restore();
}

export function link(
  ctx: CanvasRenderingContext2D,
  from: Px,
  to: Px,
  colour: string,
  alpha: number
): void {
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.strokeStyle = colour;
  ctx.setLineDash([3, 3]);
  ctx.beginPath();
  ctx.moveTo(from[0], from[1]);
  ctx.lineTo(to[0], to[1]);
  ctx.stroke();
  ctx.restore();
}

function tracePath(ctx: CanvasRenderingContext2D, pts: Px[], rPx: number): void {
  const r = Math.max(0.5, rPx);
  const head = pts[0];
  if (!head) return;
  if (pts.length === 1) {
    ctx.beginPath();
    ctx.arc(head[0], head[1], r, 0, Math.PI * 2);
    ctx.fill();
    return;
  }
  ctx.lineWidth = r * 2;
  ctx.beginPath();
  ctx.moveTo(head[0], head[1]);
  for (const p of pts.slice(1)) ctx.lineTo(p[0], p[1]);
  ctx.stroke();
}

export type PathStyle = { fill: string; fillAlpha: number; line: string; lineAlpha: number };

export function paintPath(
  ctx: CanvasRenderingContext2D,
  layer: HTMLCanvasElement,
  pts: Px[],
  rPx: number,
  style: PathStyle
): void {
  const w = ctx.canvas.width;
  const h = ctx.canvas.height;
  ctx.save();
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  ctx.globalAlpha = style.fillAlpha;
  ctx.fillStyle = style.fill;
  ctx.strokeStyle = style.fill;
  tracePath(ctx, pts, rPx);
  ctx.restore();

  if (layer.width !== w) layer.width = w;
  if (layer.height !== h) layer.height = h;
  const lc = layer.getContext('2d');
  if (!lc) return;
  lc.clearRect(0, 0, w, h);
  lc.lineCap = 'round';
  lc.lineJoin = 'round';
  lc.fillStyle = style.line;
  lc.strokeStyle = style.line;
  tracePath(lc, pts, rPx + 0.75);
  lc.globalCompositeOperation = 'destination-out';
  tracePath(lc, pts, rPx - 0.75);
  lc.globalCompositeOperation = 'source-over';

  ctx.save();
  ctx.globalAlpha = style.lineAlpha;
  ctx.drawImage(layer, 0, 0);
  ctx.restore();
}

export function distToPolyline(pts: Px[], x: number, y: number): number {
  let best = Infinity;
  for (let i = 0; i < pts.length; i++) {
    const a = pts[i];
    const b = pts[i + 1] ?? a;
    if (!a || !b) continue;
    const dx = b[0] - a[0];
    const dy = b[1] - a[1];
    const len2 = dx * dx + dy * dy;
    const t = len2 > 0 ? Math.max(0, Math.min(1, ((x - a[0]) * dx + (y - a[1]) * dy) / len2)) : 0;
    best = Math.min(best, Math.hypot(x - (a[0] + dx * t), y - (a[1] + dy * t)));
  }
  return best;
}
