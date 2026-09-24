import { describe, expect, it } from 'vitest';
import { RenderHost, type Port, type PortEvents } from './host';
import type { Reply, Request } from './protocol';

class FakePort implements Port {
  sent: { message: Request; transfer: Transferable[] }[] = [];
  terminated = false;
  private listeners = new Map<string, (event: unknown) => void>();

  postMessage(message: Request, transfer: Transferable[]): void {
    this.sent.push({ message, transfer });
  }

  addEventListener<K extends keyof PortEvents>(
    type: K,
    listener: (event: PortEvents[K]) => void
  ): void {
    this.listeners.set(type, listener as (event: unknown) => void);
  }

  terminate(): void {
    this.terminated = true;
  }

  reply(reply: Reply): void {
    this.listeners.get('message')?.({ data: reply });
  }

  crash(message: string): void {
    this.listeners.get('error')?.({ message });
  }
}

async function started(port: FakePort): Promise<RenderHost> {
  const pending = RenderHost.start(() => port);
  port.reply({ id: 0, ok: true, value: 'gpu' });
  return pending;
}

describe('render host', () => {
  it('starts the worker and keeps the adapter label', async () => {
    const port = new FakePort();
    const host = await started(port);
    expect(host.adapter).toBe('gpu');
    expect(port.sent.map((s) => s.message.call.op)).toEqual(['init']);
  });

  it('matches replies to calls by id and transfers buffers', async () => {
    const port = new FakePort();
    const host = await started(port);
    const bytes = new ArrayBuffer(8);
    const source = host.setSource(bytes);
    const frame = host.render({} as never, { max_edge: 256 });
    port.reply({ id: 2, ok: true, value: { width: 2 } });
    port.reply({ id: 1, ok: true, value: { width: 1 } });

    expect(await frame).toEqual({ width: 2 });
    expect(await source).toEqual({ width: 1 });
    expect(port.sent.map((s) => s.transfer)).toEqual([[], [bytes], []]);
  });

  it('rejects with the worker error', async () => {
    const port = new FakePort();
    const host = await started(port);
    const lut = host.setLut('l', new ArrayBuffer(1));
    port.reply({ id: 1, ok: false, error: 'lut l: bad size' });
    await expect(lut).rejects.toThrow('lut l: bad size');
  });

  it('terminates the worker when init fails', async () => {
    const port = new FakePort();
    const pending = RenderHost.start(() => port);
    port.reply({ id: 0, ok: false, error: 'no adapter' });
    await expect(pending).rejects.toThrow('no adapter');
    expect(port.terminated).toBe(true);
  });

  it('rejects outstanding calls when the worker crashes', async () => {
    const port = new FakePort();
    const host = await started(port);
    const frame = host.render({} as never, { max_edge: 256 });
    port.crash('wasm trap');
    await expect(frame).rejects.toThrow('wasm trap');
  });

  it('rejects outstanding calls on dispose', async () => {
    const port = new FakePort();
    const host = await started(port);
    const frame = host.render({} as never, { max_edge: 256 });
    host.dispose();
    await expect(frame).rejects.toThrow('disposed');
    expect(port.terminated).toBe(true);
  });
});
