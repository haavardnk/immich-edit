export type SlotResult = { ok: true } | { ok: false; error: unknown };

export interface SlotWrite<V> {
  from: V;
  to: V;
  show: (value: V) => void;
  write: (saved: V) => Promise<unknown>;
}

export interface SlotQueue<V> {
  write(key: string, change: SlotWrite<V>): Promise<SlotResult>;
}

interface Slot<V> {
  tail: Promise<unknown>;
  latest: number;
  pending: number;
  saved: V;
  show: (value: V) => void;
}

export function slotQueue<V>(): SlotQueue<V> {
  const slots = new Map<string, Slot<V>>();
  let sequence = 0;

  async function settle(key: string, slot: Slot<V>, result: SlotResult): Promise<SlotResult> {
    slot.pending -= 1;
    if (slot.pending > 0) return result.ok ? result : { ok: true };
    slots.delete(key);
    if (result.ok) return result;
    const restored = display(slot.show, slot.saved);
    return restored.ok ? result : restored;
  }

  return {
    write(key, change) {
      const id = ++sequence;
      const slot = slots.get(key) ?? {
        tail: Promise.resolve(),
        latest: id,
        pending: 0,
        saved: change.from,
        show: change.show
      };
      slots.set(key, slot);
      slot.latest = id;
      slot.pending += 1;
      slot.show = change.show;
      const shown = display(change.show, change.to);
      const run = slot.tail.then(async () => {
        if (slot.latest !== id) return settle(key, slot, { ok: true });
        const result = shown.ok ? await attempt(() => change.write(slot.saved)) : shown;
        if (result.ok) slot.saved = change.to;
        return settle(key, slot, result);
      });
      slot.tail = run;
      return run;
    }
  };
}

function display<V>(show: (value: V) => void, value: V): SlotResult {
  try {
    show(value);
    return { ok: true };
  } catch (error) {
    return { ok: false, error };
  }
}

async function attempt(write: () => Promise<unknown>): Promise<SlotResult> {
  try {
    await write();
    return { ok: true };
  } catch (error) {
    return { ok: false, error };
  }
}
