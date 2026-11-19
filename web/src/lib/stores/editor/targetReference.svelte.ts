import { livePreview } from '#lib/api/preview.ts';
import { averageAt, type PixelGrid, type Rgb } from '#lib/editor/targetPick.ts';
import type { Edits } from '#lib/types/edits.ts';
import { SingleFlight } from '#lib/utils/singleFlight.ts';

const REFERENCE_EDGE = 768;

export interface ReferenceSource {
  assetId: string;
  edits: Edits;
}

interface ReferenceArgs extends ReferenceSource {
  key: string;
}

async function decode(blob: Blob): Promise<PixelGrid | null> {
  const bitmap = await createImageBitmap(blob);
  const canvas = document.createElement('canvas');
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  const context = canvas.getContext('2d', { willReadFrequently: true });
  context?.drawImage(bitmap, 0, 0);
  bitmap.close();
  if (!context || canvas.width < 1 || canvas.height < 1) return null;
  const { data } = context.getImageData(0, 0, canvas.width, canvas.height);
  return { width: canvas.width, height: canvas.height, data };
}

export class TargetReference {
  grid = $state.raw<PixelGrid | null>(null);
  failed = $state(false);
  private key = '';
  private assetId = '';
  private flight = new SingleFlight<ReferenceArgs, PixelGrid | null>(
    async ({ assetId, edits }, signal) => {
      const { blob } = await livePreview(
        assetId,
        edits,
        REFERENCE_EDGE,
        'none',
        undefined,
        signal,
        'reference'
      );
      return decode(blob);
    },
    ({ key }, grid) => {
      if (key === this.key) this.grid = grid;
    },
    () => {
      this.failed = true;
    }
  );

  sync(source: ReferenceSource | null): void {
    if (!source) {
      this.clear();
      return;
    }
    const key = JSON.stringify(source);
    if (key === this.key) return;
    this.key = key;
    if (source.assetId !== this.assetId) this.grid = null;
    this.assetId = source.assetId;
    this.failed = false;
    this.flight.submit({ ...source, key });
  }

  sample(u: number, v: number): Rgb | null {
    return this.grid ? averageAt(this.grid, u, v) : null;
  }

  clear(): void {
    this.key = '';
    this.assetId = '';
    this.flight.cancel();
    this.grid = null;
    this.failed = false;
  }
}
