import { describe, expect, it } from 'vitest';
import { slotQueue, type SlotResult } from './metaSlots';

interface Held {
  done: Promise<SlotResult>;
  saved: () => number | undefined;
  resolve: () => void;
  reject: () => void;
}

function rating(start: number): { shown: () => number; write: (to: number) => Held } {
  const queue = slotQueue<number>();
  let value = start;
  return {
    shown: () => value,
    write: (to) => {
      let resolve: () => void = () => {};
      let reject: () => void = () => {};
      let saved: number | undefined;
      const call = new Promise<void>((res, rej) => {
        resolve = res;
        reject = () => rej(new Error('refused'));
      });
      const done = queue.write('a:rating', {
        from: value,
        to,
        show: (v) => (value = v),
        write: (confirmed) => {
          saved = confirmed;
          return call;
        }
      });
      return { done, saved: () => saved, resolve, reject };
    }
  };
}

async function flush(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

describe('slotQueue', () => {
  it('shows the new value at once and keeps it when the write lands', async () => {
    const slot = rating(2);
    const write = slot.write(4);
    expect(slot.shown()).toBe(4);
    await flush();
    write.resolve();
    expect(await write.done).toEqual({ ok: true });
    expect(slot.shown()).toBe(4);
  });

  it('restores the old value when the only write fails', async () => {
    const slot = rating(2);
    const write = slot.write(4);
    await flush();
    write.reject();
    expect((await write.done).ok).toBe(false);
    expect(slot.shown()).toBe(2);
  });

  it('sends only the last of several changes made together', async () => {
    const slot = rating(2);
    const older = slot.write(3);
    const newer = slot.write(4);
    await flush();
    expect(older.saved()).toBeUndefined();
    expect(newer.saved()).toBe(2);
    newer.resolve();
    expect(await Promise.all([older.done, newer.done])).toEqual([{ ok: true }, { ok: true }]);
    expect(slot.shown()).toBe(4);
  });

  it('sends a newer change after the write in flight, from the saved value', async () => {
    const slot = rating(2);
    const older = slot.write(3);
    await flush();
    const newer = slot.write(4);
    await flush();
    expect(newer.saved()).toBeUndefined();
    older.resolve();
    await older.done;
    await flush();
    expect(newer.saved()).toBe(3);
    newer.resolve();
    await newer.done;
    expect(slot.shown()).toBe(4);
  });

  it('does not report an older failure a newer write replaced', async () => {
    const slot = rating(2);
    const older = slot.write(3);
    await flush();
    const newer = slot.write(4);
    older.reject();
    expect(await older.done).toEqual({ ok: true });
    expect(slot.shown()).toBe(4);
    await flush();
    expect(newer.saved()).toBe(2);
    newer.resolve();
    await newer.done;
    expect(slot.shown()).toBe(4);
  });

  it('falls back to the last saved value when the newest write fails', async () => {
    const slot = rating(2);
    const older = slot.write(3);
    await flush();
    const newer = slot.write(4);
    older.resolve();
    await older.done;
    await flush();
    newer.reject();
    expect((await newer.done).ok).toBe(false);
    expect(slot.shown()).toBe(3);
  });

  it('starts fresh once a slot has settled', async () => {
    const slot = rating(2);
    const first = slot.write(3);
    await flush();
    first.resolve();
    await first.done;
    const second = slot.write(5);
    await flush();
    expect(second.saved()).toBe(3);
    second.reject();
    await second.done;
    expect(slot.shown()).toBe(3);
  });

  it('fails without writing when the value cannot be shown', async () => {
    const queue = slotQueue<number>();
    let writes = 0;
    const shown: number[] = [];
    const result = await queue.write('a:rating', {
      from: 2,
      to: 4,
      show: (v) => {
        if (v === 4) throw new Error('detached');
        shown.push(v);
      },
      write: () => {
        writes += 1;
        return Promise.resolve();
      }
    });
    expect(result.ok).toBe(false);
    expect(writes).toBe(0);
    expect(shown).toEqual([2]);
    const next = await queue.write('a:rating', {
      from: 2,
      to: 3,
      show: (v) => shown.push(v),
      write: () => Promise.resolve()
    });
    expect(next).toEqual({ ok: true });
  });
});
