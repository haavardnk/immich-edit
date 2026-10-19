import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { GpuLimits, TickSample } from '$lib/render/protocol';

const storageKey = 'immich-edit:renderer';
const NEED = { benchmarkEdge: 1600, maxEdge: 2560 };
const ROOMY: GpuLimits = { max_texture_dimension_2d: 8192, max_buffer_size: 1 << 30 };

const gpu = vi.hoisted(() => ({
  start: vi.fn(),
  dispose: vi.fn()
}));

vi.mock('$lib/api/clientLog', () => ({ reportClientError: vi.fn() }));
vi.mock('$lib/render/clientRenderer', () => ({ ClientRenderer: { start: gpu.start } }));

function fakeClient(benchmark: TickSample | null, limits: GpuLimits = ROOMY) {
  return { adapter: 'Fake GPU', limits, initMs: 120, benchmark, dispose: gpu.dispose };
}

function stubStorage(initial?: unknown): Map<string, string> {
  const values = new Map<string, string>();
  if (initial !== undefined) values.set(storageKey, JSON.stringify(initial));
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value)
  });
  return values;
}

async function load(stored?: unknown) {
  const values = stubStorage(stored);
  const { renderer } = await import('./renderer.svelte');
  return { renderer, values };
}

function flush(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

const TEN_MS_PER_MP: TickSample = { ms: 16, pixels: 1_600_000 };

beforeEach(() => {
  gpu.start.mockReset();
  gpu.dispose.mockReset();
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe('renderer speed gate', () => {
  it('picks the server when the server is faster', async () => {
    gpu.start.mockResolvedValue(fakeClient(TEN_MS_PER_MP));
    const { renderer } = await load({ choice: 'auto', serverMs: 5 });

    expect(renderer.bind(NEED).kind).toBe('server');
    await flush();

    expect(renderer.bind(NEED).kind).toBe('server');
    expect(renderer.state).toBe('server');
    expect(renderer.browserMs).toBe(10);
    expect(renderer.reason).toBe('browser not clearly faster, 10.0 vs 5.0 ms/MP');
  });

  it('picks the browser when the browser is faster', async () => {
    gpu.start.mockResolvedValue(fakeClient(TEN_MS_PER_MP));
    const { renderer } = await load({ choice: 'auto', serverMs: 40 });

    expect(renderer.bind(NEED).kind).toBe('server');
    await flush();

    const binding = renderer.bind(NEED);
    expect(binding.kind).toBe('browser');
    expect(renderer.state).toBe('browser');
    expect(renderer.reason).toBe('browser faster, 10.0 vs 40.0 ms/MP');
  });

  it('picks the server when the browser is faster only within the margin', async () => {
    gpu.start.mockResolvedValue(fakeClient(TEN_MS_PER_MP));
    const { renderer } = await load({ choice: 'auto', serverMs: 11 });

    renderer.bind(NEED);
    await flush();

    expect(renderer.bind(NEED).kind).toBe('server');
    expect(renderer.state).toBe('server');
  });

  it('picks the server when the GPU limits are too small', async () => {
    gpu.start.mockResolvedValue(
      fakeClient(TEN_MS_PER_MP, { max_texture_dimension_2d: 2048, max_buffer_size: 1 << 30 })
    );
    const { renderer } = await load({ choice: 'auto', serverMs: 40 });

    renderer.bind(NEED);
    await flush();

    expect(renderer.bind(NEED).kind).toBe('server');
    expect(renderer.reason).toBe('GPU textures max 2048 px, need 2560 px');
  });

  it('picks the server when the GPU buffers are too small', async () => {
    gpu.start.mockResolvedValue(
      fakeClient(TEN_MS_PER_MP, { max_texture_dimension_2d: 8192, max_buffer_size: 1 << 20 })
    );
    const { renderer } = await load({ choice: 'auto', serverMs: 40 });

    renderer.bind(NEED);
    await flush();

    expect(renderer.bind(NEED).kind).toBe('server');
    expect(renderer.reason).toBe('GPU buffers max 1.0MiB, need 50.0MiB');
  });

  it('renders on the server while measuring and decides from real ticks', async () => {
    gpu.start.mockResolvedValue(fakeClient(TEN_MS_PER_MP));
    const { renderer, values } = await load();

    expect(renderer.bind(NEED).kind).toBe('server');
    expect(renderer.state).toBe('starting');
    await flush();
    expect(renderer.state).toBe('measuring');
    expect(renderer.bind(NEED).kind).toBe('server');

    renderer.recordServer({ ms: 48, pixels: 1_600_000 });
    expect(renderer.serverMs).toBe(30);
    expect(renderer.state).toBe('browser');
    expect(JSON.parse(values.get(storageKey) ?? '{}')).toEqual({
      choice: 'auto',
      browserMs: 10,
      serverMs: 30
    });

    expect(renderer.bind(NEED).kind).toBe('browser');
  });

  it('feeds real ticks into a moving average', async () => {
    gpu.start.mockResolvedValue(fakeClient(TEN_MS_PER_MP));
    const { renderer } = await load({ choice: 'auto', serverMs: 30 });
    renderer.bind(NEED);
    await flush();

    expect(renderer.state).toBe('browser');
    renderer.recordRender({ ms: 112, pixels: 1_600_000 });

    expect(renderer.lastRenderMs).toBe(112);
    expect(renderer.browserMs).toBeCloseTo(28);
    expect(renderer.state).toBe('server');
  });

  it('waits for the browser without a benchmark when the remembered verdict picks it', async () => {
    gpu.start.mockResolvedValue(fakeClient(null));
    const { renderer } = await load({ choice: 'auto', browserMs: 8, serverMs: 40 });

    const binding = renderer.bind(NEED);
    expect(binding.kind).toBe('wait');
    if (binding.kind === 'wait') await binding.ready;

    expect(renderer.bind(NEED).kind).toBe('browser');
    expect(gpu.start).toHaveBeenCalledOnce();
    expect(gpu.start).toHaveBeenCalledWith(null);
    expect(renderer.browserMs).toBe(8);
  });

  it('forces the browser when browser rendering is selected', async () => {
    gpu.start.mockResolvedValue(fakeClient(null));
    const { renderer } = await load({ choice: 'browser', serverMs: 1 });

    const binding = renderer.bind(NEED);
    expect(binding.kind).toBe('wait');
    if (binding.kind === 'wait') await binding.ready;

    expect(renderer.bind(NEED).kind).toBe('browser');
    expect(renderer.state).toBe('browser');
    expect(renderer.reason).toBe('browser selected');
    expect(gpu.start).toHaveBeenCalledWith(null);
  });

  it('never starts the browser renderer when server rendering is selected', async () => {
    const { renderer } = await load({ choice: 'server' });

    expect(renderer.bind(NEED).kind).toBe('server');
    expect(renderer.state).toBe('server');
    expect(renderer.reason).toBe('server selected');
    expect(gpu.start).not.toHaveBeenCalled();
  });

  it('stays on the server after a failure until the choice changes', async () => {
    gpu.start.mockRejectedValue(new Error('no WebGPU'));
    const { renderer } = await load({ choice: 'auto', browserMs: 1, serverMs: 40 });
    expect(renderer.measuresServer).toBe(true);

    const binding = renderer.bind(NEED);
    if (binding.kind === 'wait') await binding.ready;
    expect(renderer.bind(NEED).kind).toBe('server');
    expect(renderer.reason).toBe('no WebGPU');
    expect(gpu.start).toHaveBeenCalledOnce();
    expect(renderer.measuresServer).toBe(false);

    gpu.start.mockResolvedValue(fakeClient(TEN_MS_PER_MP));
    renderer.setChoice('browser');
    expect(renderer.state).toBe('idle');
    renderer.bind(NEED);
    expect(gpu.start).toHaveBeenCalledTimes(2);
  });

  it('drops the browser renderer when it fails mid-session', async () => {
    gpu.start.mockResolvedValue(fakeClient(TEN_MS_PER_MP));
    const { renderer } = await load({ choice: 'browser' });
    const binding = renderer.bind(NEED);
    if (binding.kind === 'wait') await binding.ready;

    renderer.fail(new Error('device lost'));

    expect(gpu.dispose).toHaveBeenCalledOnce();
    expect(renderer.current).toBeNull();
    expect(renderer.reason).toBe('browser failed: device lost');
    expect(renderer.bind(NEED).kind).toBe('server');
  });

  it('ignores unknown stored values', async () => {
    const { renderer } = await load({ choice: 'gpu', browserMs: -1, serverMs: 'fast' });

    expect(renderer.choice).toBe('auto');
    expect(renderer.browserMs).toBeNull();
    expect(renderer.serverMs).toBeNull();
  });
});
