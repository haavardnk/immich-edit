import type { MaskRange } from '$lib/api/masks';
import { fetchRaster } from '$lib/api/rasters';
import type { BrushBuffer } from '$lib/utils/brush';
import { windowLevels } from '$lib/utils/depthWindow';
import type { MaskStroke } from '$lib/stores/editor/maskWeight.svelte';

export interface DepthPreviewCtx {
  brushBuffers: Record<string, BrushBuffer>;
  brushBufferSource: Record<string, string>;
  maskStroke: MaskStroke;
  endMaskStroke(): void;
}

export interface DepthTarget {
  componentId: string;
  rasterId: string;
  probRasterId: string;
  range: MaskRange;
}

interface Session {
  componentId: string;
  original: BrushBuffer | undefined;
  source: string | undefined;
  shown: BrushBuffer | null;
}

export class DepthPreview {
  private readonly ctx: DepthPreviewCtx;
  private levels: { id: string; raster: Promise<BrushBuffer> } | null = null;
  private wanted: DepthTarget | null = null;
  private session: Session | null = null;
  private frame: number | null = null;

  constructor(ctx: DepthPreviewCtx) {
    this.ctx = ctx;
  }

  show(target: DepthTarget): void {
    this.wanted = target;
    this.frame ??= requestAnimationFrame(() => void this.paint());
  }

  end(): void {
    this.stop();
    const session = this.session;
    this.session = null;
    if (!session) return;
    const id = session.componentId;
    if (session.shown && this.ctx.brushBuffers[id] === session.shown) this.restore(session);
    if (this.ctx.maskStroke === 'painting') this.ctx.endMaskStroke();
  }

  reset(): void {
    this.stop();
    this.session = null;
    this.levels = null;
  }

  private stop(): void {
    this.wanted = null;
    if (this.frame !== null) cancelAnimationFrame(this.frame);
    this.frame = null;
  }

  private restore(session: Session): void {
    const id = session.componentId;
    const buffers = { ...this.ctx.brushBuffers };
    if (session.original) buffers[id] = session.original;
    else delete buffers[id];
    this.ctx.brushBuffers = buffers;
    if (session.source === undefined) delete this.ctx.brushBufferSource[id];
    else this.ctx.brushBufferSource[id] = session.source;
  }

  private async paint(): Promise<void> {
    this.frame = null;
    const target = this.wanted;
    if (!target) return;
    const levels = await this.levelsOf(target.probRasterId);
    if (!levels || this.wanted !== target) return;
    const id = target.componentId;
    if (this.session?.componentId !== id) {
      this.session = {
        componentId: id,
        original: this.ctx.brushBuffers[id],
        source: this.ctx.brushBufferSource[id],
        shown: null
      };
    }
    const shown: BrushBuffer = {
      width: levels.width,
      height: levels.height,
      bytes: windowLevels(levels.bytes, target.range)
    };
    this.session.shown = shown;
    this.ctx.brushBuffers = { ...this.ctx.brushBuffers, [id]: shown };
    this.ctx.brushBufferSource[id] = target.rasterId;
    this.ctx.maskStroke = 'painting';
  }

  private async levelsOf(id: string): Promise<BrushBuffer | null> {
    if (this.levels?.id !== id) {
      this.levels = {
        id,
        raster: fetchRaster(id).then((r) => ({ width: r.width, height: r.height, bytes: r.bytes }))
      };
    }
    const entry = this.levels;
    try {
      return await entry.raster;
    } catch {
      if (this.levels === entry) this.levels = null;
      return null;
    }
  }
}
