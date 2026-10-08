import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

type Module = typeof import('./clientLog') & typeof import('./client');

async function load(): Promise<Module> {
  vi.resetModules();
  const log = await import('./clientLog');
  const client = await import('./client');
  return { ...log, ...client };
}

function sentBodies(fetchMock: ReturnType<typeof vi.fn>): Record<string, unknown>[] {
  return fetchMock.mock.calls.map(
    ([, init]) => JSON.parse(String((init as RequestInit).body)) as Record<string, unknown>
  );
}

describe('reportClientError', () => {
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    fetchMock = vi.fn(() => Promise.resolve(new Response(null, { status: 204 })));
    vi.stubGlobal('fetch', fetchMock);
    vi.stubGlobal('location', { pathname: '/edit/abc' });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it('posts kind, level, message, stack and route', async () => {
    const { reportClientError } = await load();
    const err = new Error('boom');
    reportClientError('render_worker', err, 'warn');
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/client-log',
      expect.objectContaining({ method: 'POST', keepalive: true })
    );
    expect(sentBodies(fetchMock)).toEqual([
      {
        kind: 'render_worker',
        level: 'warn',
        message: 'boom',
        stack: err.stack,
        route: '/edit/abc'
      }
    ]);
  });

  it.each([
    ['api error', (m: Module) => new m.ApiError(500, 'internal', 'nope')],
    ['network error', (m: Module) => new m.NetworkError('offline')],
    ['abort', () => new DOMException('aborted', 'AbortError')]
  ])('skips %s', async (_, make) => {
    const mod = await load();
    mod.reportClientError('exception', make(mod));
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('dedupes repeats for a minute and caps a page load', async () => {
    vi.useFakeTimers();
    const { reportClientError } = await load();
    reportClientError('exception', new Error('same'));
    reportClientError('exception', new Error('same'));
    expect(fetchMock).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(60_001);
    reportClientError('exception', new Error('same'));
    expect(fetchMock).toHaveBeenCalledTimes(2);
    for (let i = 0; i < 30; i++) reportClientError('exception', new Error(`e${i}`));
    expect(fetchMock).toHaveBeenCalledTimes(20);
  });
});
