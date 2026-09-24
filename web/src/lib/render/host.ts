import type { Edits } from '$lib/types/edits';
import type { Call, RenderView, RenderedFrame, Reply, Request, SourceInfo } from './protocol';

export interface Port {
  postMessage(message: Request, transfer: Transferable[]): void;
  addEventListener(type: 'message', listener: (event: MessageEvent<Reply>) => void): void;
  terminate(): void;
}

function spawnWorker(): Port {
  return new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
}

export class RenderHost {
  private readonly port: Port;
  private readonly pending = new Map<
    number,
    { resolve: (v: unknown) => void; reject: (e: Error) => void }
  >();
  private nextId = 0;
  private label = '';

  private constructor(port: Port) {
    this.port = port;
    port.addEventListener('message', ({ data }) => this.settle(data));
  }

  get adapter(): string {
    return this.label;
  }

  static async start(
    canvas: HTMLCanvasElement,
    spawn: () => Port = spawnWorker
  ): Promise<RenderHost> {
    const host = new RenderHost(spawn());
    const offscreen = canvas.transferControlToOffscreen();
    try {
      host.label = await host.call<string>({ op: 'init', canvas: offscreen }, [offscreen]);
      return host;
    } catch (err) {
      host.dispose();
      throw err;
    }
  }

  setSource(bytes: ArrayBuffer): Promise<SourceInfo> {
    return this.call({ op: 'setSource', bytes }, [bytes]);
  }

  setRaster(id: string, width: number, height: number, bytes: ArrayBuffer): Promise<void> {
    return this.call({ op: 'setRaster', id, width, height, bytes }, [bytes]);
  }

  dropRaster(id: string): Promise<void> {
    return this.call({ op: 'dropRaster', id });
  }

  setLut(id: string, bytes: ArrayBuffer): Promise<void> {
    return this.call({ op: 'setLut', id, bytes }, [bytes]);
  }

  dropLut(id: string): Promise<void> {
    return this.call({ op: 'dropLut', id });
  }

  setDcp(bytes: ArrayBuffer | null): Promise<void> {
    return this.call({ op: 'setDcp', bytes }, bytes ? [bytes] : []);
  }

  render(edits: Edits, view: RenderView): Promise<RenderedFrame> {
    return this.call({ op: 'render', edits, view });
  }

  dispose(): void {
    this.port.terminate();
    for (const { reject } of this.pending.values()) reject(new Error('the renderer was disposed'));
    this.pending.clear();
  }

  private call<T>(call: Call, transfer: Transferable[] = []): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      this.port.postMessage({ id, call }, transfer);
    });
  }

  private settle(reply: Reply): void {
    const waiter = this.pending.get(reply.id);
    if (!waiter) return;
    this.pending.delete(reply.id);
    if (reply.ok) waiter.resolve(reply.value);
    else waiter.reject(new Error(reply.error));
  }
}
