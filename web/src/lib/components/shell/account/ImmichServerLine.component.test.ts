import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import ImmichServerLine from './ImmichServerLine.svelte';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(host: string | null, version: string | null): string {
  immichCapabilities.externalHost = host;
  immichCapabilities.version = version;
  component = mount(ImmichServerLine, { target, props: { class: 'line' } });
  flushSync();
  return target.textContent ?? '';
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  immichCapabilities.externalHost = null;
  immichCapabilities.version = null;
});

describe('ImmichServerLine', () => {
  it('names the public host and the Immich version', () => {
    expect(render('photos.example.net', '3.2.2')).toBe('photos.example.net · Immich v3.2.2');
    expect(target.querySelector('p.line')).not.toBeNull();
  });

  it('names only the host when the version is unknown', () => {
    expect(render('photos.example.net:8443', null)).toBe('photos.example.net:8443');
  });

  it('renders nothing without a public host', () => {
    expect(render(null, '3.2.2')).toBe('');
    expect(target.querySelector('p')).toBeNull();
  });
});
