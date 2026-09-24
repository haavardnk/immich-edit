import type { Call } from './protocol';

export interface Renderer {
  adapter(): string;
  set_source(bytes: Uint8Array): unknown;
  set_raster(id: string, width: number, height: number, bytes: Uint8Array): void;
  drop_raster(id: string): void;
  set_lut(id: string, bytes: Uint8Array): void;
  drop_lut(id: string): void;
  set_dcp(bytes: Uint8Array | undefined): void;
  render(edits: string, view: string): Promise<unknown>;
}

export type Load = (canvas: OffscreenCanvas) => Promise<Renderer>;

export function createDispatcher(load: Load): (call: Call) => Promise<unknown> {
  let renderer: Renderer | null = null;
  let queue: Promise<unknown> = Promise.resolve();

  const run = async (call: Call): Promise<unknown> => {
    if (call.op === 'init') {
      renderer = await load(call.canvas);
      return renderer.adapter();
    }
    if (!renderer) throw new Error('the renderer has not been initialised');
    switch (call.op) {
      case 'setSource':
        return renderer.set_source(new Uint8Array(call.bytes));
      case 'setRaster':
        return renderer.set_raster(call.id, call.width, call.height, new Uint8Array(call.bytes));
      case 'dropRaster':
        return renderer.drop_raster(call.id);
      case 'setLut':
        return renderer.set_lut(call.id, new Uint8Array(call.bytes));
      case 'dropLut':
        return renderer.drop_lut(call.id);
      case 'setDcp':
        return renderer.set_dcp(call.bytes ? new Uint8Array(call.bytes) : undefined);
      case 'render':
        return renderer.render(JSON.stringify(call.edits), JSON.stringify(call.view));
    }
  };

  return (call) => {
    const next = queue.then(() => run(call));
    queue = next.catch(() => undefined);
    return next;
  };
}
