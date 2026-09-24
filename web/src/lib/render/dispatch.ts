import type { Call } from './protocol';

export interface Renderer {
  adapter(): string;
  set_source(bytes: Uint8Array): unknown;
  set_tile(bytes: Uint8Array): unknown;
  drop_tile(): void;
  set_raster(id: string, width: number, height: number, bytes: Uint8Array): void;
  drop_raster(id: string): void;
  set_lut(id: string, bytes: Uint8Array): void;
  drop_lut(id: string): void;
  set_dcp(bytes: Uint8Array | undefined): void;
  render(edits: string, view: string): Promise<unknown>;
}

export interface Wasm {
  renderer: Renderer;
  inputs(edits: string, maxEdge: number): unknown;
}

export type Load = () => Promise<Wasm>;

export interface Outcome {
  value: unknown;
  transfer: Transferable[];
}

export function createDispatcher(load: Load): (call: Call) => Promise<Outcome> {
  let wasm: Wasm | null = null;
  let queue: Promise<unknown> = Promise.resolve();

  const loaded = (): Wasm => {
    if (!wasm) throw new Error('the renderer has not been initialised');
    return wasm;
  };

  const run = async (call: Call): Promise<unknown> => {
    if (call.op === 'init') {
      wasm = await load();
      return wasm.renderer.adapter();
    }
    const { renderer, inputs } = loaded();
    switch (call.op) {
      case 'inputs':
        return inputs(JSON.stringify(call.edits), call.maxEdge);
      case 'setSource':
        return renderer.set_source(new Uint8Array(call.bytes));
      case 'setTile':
        return renderer.set_tile(new Uint8Array(call.bytes));
      case 'dropTile':
        return renderer.drop_tile();
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

  return async (call) => {
    if (call.op === 'inputs') return { value: await run(call), transfer: [] };
    const next = queue.then(() => run(call));
    queue = next.catch(() => undefined);
    const value = await next;
    const bitmap = call.op === 'render' ? (value as { bitmap?: ImageBitmap }).bitmap : undefined;
    return { value, transfer: bitmap ? [bitmap] : [] };
  };
}
