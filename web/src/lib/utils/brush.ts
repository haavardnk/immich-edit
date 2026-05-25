export interface BrushBuffer {
  width: number;
  height: number;
  bytes: Uint8Array;
}

export function blankBuffer(width: number, height: number): BrushBuffer {
  return { width, height, bytes: new Uint8Array(width * height) };
}

export function stampBuffer(
  buf: BrushBuffer,
  cx: number,
  cy: number,
  radius: number,
  hardness: number,
  alphaByte: number,
  erase: boolean
): void {
  const r = Math.max(0.5, radius);
  const x0 = Math.max(0, Math.floor(cx - r));
  const y0 = Math.max(0, Math.floor(cy - r));
  const x1 = Math.min(buf.width - 1, Math.ceil(cx + r));
  const y1 = Math.min(buf.height - 1, Math.ceil(cy + r));
  if (x0 > x1 || y0 > y1) return;
  const r2 = r * r;
  const h = Math.min(1, Math.max(0, hardness));
  const inner = r * h;
  const innerR2 = inner * inner;
  const falloffDen = Math.max(1e-3, r - inner);
  for (let y = y0; y <= y1; y++) {
    const dy = y - cy;
    const dy2 = dy * dy;
    const row = y * buf.width;
    for (let x = x0; x <= x1; x++) {
      const dx = x - cx;
      const d2 = dx * dx + dy2;
      if (d2 > r2) continue;
      let t = 1;
      if (d2 > innerR2) {
        const d = Math.sqrt(d2);
        const u = (r - d) / falloffDen;
        t = u * u * (3 - 2 * u);
      }
      const add = Math.round(alphaByte * t);
      const i = row + x;
      const cur = buf.bytes[i];
      buf.bytes[i] = erase ? Math.max(0, cur - add) : Math.min(255, cur + add);
    }
  }
}

export function bufferToImageData(
  buf: BrushBuffer,
  color: [number, number, number],
  intensity: number
): ImageData {
  const data = new Uint8ClampedArray(buf.width * buf.height * 4);
  const [r, g, b] = color;
  const k = Math.max(0, Math.min(1, intensity));
  for (let i = 0; i < buf.bytes.length; i++) {
    const a = buf.bytes[i];
    const j = i * 4;
    data[j] = r;
    data[j + 1] = g;
    data[j + 2] = b;
    data[j + 3] = Math.round(a * k);
  }
  return new ImageData(data, buf.width, buf.height);
}

export function parseHexColor(hex: string): [number, number, number] {
  const m = hex.replace('#', '');
  const v = m.length === 3 ? m.split('').map((c) => c + c).join('') : m;
  const n = parseInt(v, 16);
  if (Number.isNaN(n)) return [255, 60, 60];
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}
