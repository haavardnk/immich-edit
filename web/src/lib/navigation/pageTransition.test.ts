import type { OnNavigate } from '$app/navigation';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { animatePageChange } from './pageTransition';

interface FakeNavigation {
  navigation: OnNavigate;
  finish: () => void;
}

function navigate(from: string, to: string): FakeNavigation {
  let finish = (): void => undefined;
  const complete = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const at = (path: string) => ({ url: new URL(path, 'http://app.test') });
  const navigation = { from: at(from), to: at(to), complete } as unknown as OnNavigate;
  return { navigation, finish };
}

const pending: Array<() => void> = [];

beforeEach(() => {
  pending.length = 0;
  const startViewTransition = (update: () => Promise<void>) => {
    let ran = false;
    const run = (): void => {
      if (ran) return;
      ran = true;
      void update().catch(() => undefined);
    };
    pending.push(run);
    const done = Promise.resolve();
    return {
      ready: done,
      updateCallbackDone: done,
      finished: done,
      skipTransition: run
    };
  };
  vi.stubGlobal('document', { startViewTransition, documentElement: { dataset: {} } });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

async function state(promise: Promise<void> | undefined): Promise<'resolved' | 'waiting'> {
  if (!promise) return 'resolved';
  const marker = Symbol('waiting');
  const winner = await Promise.race([promise.then(() => 'resolved' as const), tick(marker)]);
  return winner === marker ? 'waiting' : 'resolved';
}

async function tick<T>(value: T): Promise<T> {
  for (let i = 0; i < 20; i++) await Promise.resolve();
  return value;
}

describe('animatePageChange', () => {
  it('lets a newer navigation apply only after the one it interrupted', async () => {
    const search = navigate('/photos', '/search?q=DSC');
    const searching = animatePageChange(search.navigation, false);
    expect(await state(searching)).toBe('waiting');

    const back = navigate('/search?q=DSC', '/photos');
    const returning = animatePageChange(back.navigation, false);

    expect(await state(searching)).toBe('resolved');
    pending.forEach((run) => run());
    expect(await state(returning)).toBe('waiting');
    search.finish();
    expect(await state(returning)).toBe('resolved');
    back.finish();
    await tick(null);
  });

  it('leaves a navigation alone once the previous one has settled', async () => {
    const first = navigate('/photos', '/albums');
    const opened = animatePageChange(first.navigation, false);
    pending.forEach((run) => run());
    expect(await state(opened)).toBe('resolved');
    first.finish();
    await tick(null);

    const next = navigate('/albums', '/albums');
    expect(animatePageChange(next.navigation, false)).toBeUndefined();
    next.finish();
  });
});
