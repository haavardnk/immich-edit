import { generateDepth } from '$lib/api/depth';
import { errorMessage } from '$lib/utils/errors';
import { SvelteMap, SvelteSet } from 'svelte/reactivity';

type Apply = (rasterId: string) => Promise<boolean>;

export class DepthMapState {
  private readonly pending = new SvelteSet<string>();
  private readonly failures = new SvelteMap<string, string>();
  private readonly finished = new SvelteMap<string, string>();

  busy(assetId: string | null): boolean {
    return assetId !== null && this.pending.has(assetId);
  }

  error(assetId: string | null): string | null {
    return assetId === null ? null : (this.failures.get(assetId) ?? null);
  }

  take(assetId: string): string | null {
    const rasterId = this.finished.get(assetId);
    if (rasterId === undefined) return null;
    this.finished.delete(assetId);
    return rasterId;
  }

  generate = async (assetId: string, apply: Apply): Promise<void> => {
    if (this.busy(assetId)) return;
    this.pending.add(assetId);
    this.failures.delete(assetId);
    try {
      const depth = await generateDepth(assetId);
      if (!(await apply(depth.raster_id))) {
        this.finished.set(assetId, depth.raster_id);
      }
    } catch (e) {
      this.failures.set(assetId, errorMessage(e));
    } finally {
      this.pending.delete(assetId);
    }
  };
}

export const depthMap = new DepthMapState();
