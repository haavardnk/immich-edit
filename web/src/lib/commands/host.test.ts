import { describe, expect, it } from 'vitest';
import { createHost } from './host';

describe('command host', () => {
  it('exposes the attached implementation until it detaches', () => {
    const host = createHost<{ name: string }>();
    expect(host.get()).toBeNull();
    const detach = host.attach({ name: 'grid' });
    expect(host.get()).toEqual({ name: 'grid' });
    detach();
    expect(host.get()).toBeNull();
  });

  it('keeps a newer implementation when an older one detaches', () => {
    const host = createHost<{ name: string }>();
    const detachOld = host.attach({ name: 'old' });
    host.attach({ name: 'new' });
    detachOld();
    expect(host.get()).toEqual({ name: 'new' });
  });
});
