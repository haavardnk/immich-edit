import { HANDLE_HIT } from '#lib/editor/handles.ts';
import type { MaskComponent, MaskComponentKind, Vec2f } from '#lib/types/edits.ts';
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import MaskLinearHandles from './MaskLinearHandles.svelte';
import MaskRadialHandles from './MaskRadialHandles.svelte';

const RECT = { x: 0, y: 0, w: 400, h: 400 };

let target: SVGSVGElement;
let component: Record<string, unknown> | null = null;

beforeEach(() => {
  target = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
});

function toPx(v: Vec2f): { x: number; y: number } {
  return { x: v.x * RECT.w, y: v.y * RECT.h };
}

function comp(kind: MaskComponentKind): MaskComponent {
  return { id: 'c', enabled: true, mode: 'add', invert: false, kind, source: 'manual' };
}

function hitCircles(): { label: string; r: number }[] {
  return [...target.querySelectorAll('circle[role="button"]')].map((el) => ({
    label: el.getAttribute('aria-label') ?? '',
    r: Number(el.getAttribute('r'))
  }));
}

describe('mask handle hit areas', () => {
  it('draws the radial centre last and keeps the feather handle beside it', () => {
    const kind = {
      kind: 'radial',
      center: { x: 0.5, y: 0.5 },
      radius_xy: { x: 0.25, y: 0.25 },
      feather: 1
    } as const;
    component = mount(MaskRadialHandles, {
      target,
      props: {
        comp: comp(kind),
        kind,
        rect: RECT,
        toPx,
        aspect: 1,
        onSelect: () => {},
        onDrag: () => {}
      }
    });
    flushSync();
    const circles = hitCircles();
    expect(circles.at(-1)?.label).toBe('Radial center');
    const feather = circles.find((c) => c.label === 'Radial feather');
    const edge = circles.find((c) => c.label === 'Radial radius y');
    expect(feather?.r).toBeLessThan(HANDLE_HIT);
    expect(circles.at(-1)?.r).toBeLessThan(HANDLE_HIT);
    expect(edge?.r).toBe(HANDLE_HIT);
  });

  it('draws the linear move handle above the feather handles', () => {
    const kind = {
      kind: 'linear',
      p0: { x: 0.2, y: 0.5 },
      p1: { x: 0.8, y: 0.5 },
      feather: 0
    } as const;
    component = mount(MaskLinearHandles, {
      target,
      props: {
        comp: comp(kind),
        kind,
        rect: RECT,
        toPx,
        fromPx: (x: number, y: number) => ({ x: x / RECT.w, y: y / RECT.h }),
        onSelect: () => {},
        onDrag: () => {}
      }
    });
    flushSync();
    const labels = hitCircles().map((c) => c.label);
    expect(labels.at(-1)).toBe('Move linear');
    expect(labels.filter((l) => l === 'Linear feather')).toHaveLength(2);
  });
});
