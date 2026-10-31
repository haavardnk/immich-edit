import type { ServerTiming } from '#lib/api/serverTiming.ts';
import type { TickSample } from '#lib/render/protocol.ts';

export async function measureServerTick(
  blob: Blob,
  timing: ServerTiming
): Promise<TickSample | null> {
  try {
    const started = performance.now();
    const bitmap = await createImageBitmap(blob);
    const decodeMs = performance.now() - started;
    const pixels = bitmap.width * bitmap.height;
    bitmap.close();
    return {
      ms: timing.renderMs + timing.encodeMs + timing.transferMs + decodeMs,
      pixels
    };
  } catch {
    return null;
  }
}
