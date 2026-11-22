import { MapTrayState } from '#lib/map/mapTray.svelte.ts';
import { describe, expect, it } from 'vitest';

describe('map tray', () => {
  it('opens at half height, carries while dragging and closes', () => {
    const tray = new MapTrayState();
    tray.carry();
    expect(tray.state).toBe('closed');
    tray.detent = 1;
    tray.open();
    expect(tray).toMatchObject({ state: 'open', detent: 0 });
    tray.detent = 1;
    tray.open();
    expect(tray.detent).toBe(1);
    tray.carry();
    expect(tray.state).toBe('carrying');
    tray.close();
    expect(tray.state).toBe('closed');
  });
});
