import { describe, expect, it } from 'vitest';

const sources = import.meta.glob<string>('/src/**/*.{ts,svelte}', {
  query: '?raw',
  import: 'default',
  eager: true
});

const OWNERS = ['/src/lib/commands/', '/src/lib/shortcuts/'];

const CHORD_CHECKS = [
  '/src/lib/components/editor/ClickCanvas.svelte',
  '/src/lib/components/editor/MaskOverlay.svelte',
  '/src/lib/components/editor/MaskPolygonDraft.svelte',
  '/src/lib/components/shell/KeybindsHelp.svelte'
];

const WINDOW_KEY_LISTENERS = [
  '/src/lib/components/browse/AssetGrid.svelte',
  '/src/lib/components/editor/ClickCanvas.svelte',
  '/src/lib/components/editor/MaskOverlay.svelte',
  '/src/lib/components/editor/MaskPolygonDraft.svelte',
  '/src/lib/components/editor/WhiteBalancePicker.svelte',
  '/src/lib/components/editor/controls/SliderRow.svelte',
  '/src/lib/components/shell/KeybindsHelp.svelte',
  '/src/lib/device/layout.svelte.ts',
  '/src/lib/utils/pointerFocus.ts'
];

function dispatchesBinds(source: string): boolean {
  return /\bmatchKeybind\s*\(/.test(source);
}

function checksChords(source: string): boolean {
  return /\bisKeybind\s*\(/.test(source);
}

function listensForKeys(source: string): boolean {
  return (
    /<svelte:(window|document|body)\b[^>]*\bonkey(down|up)/.test(source) ||
    /\b(window|document)\.addEventListener\(\s*['"]key(down|up)['"]/.test(source)
  );
}

function offenders(matches: (source: string) => boolean): string[] {
  return Object.entries(sources)
    .filter(([path]) => !path.endsWith('.test.ts'))
    .filter(([path]) => !OWNERS.some((owner) => path.startsWith(owner)))
    .filter(([, source]) => matches(source))
    .map(([path]) => path)
    .sort();
}

describe('shortcut dispatch', () => {
  it('resolves shortcuts only through the command registry', () => {
    expect(Object.keys(sources).length).toBeGreaterThan(100);
    expect(offenders(dispatchesBinds)).toEqual([]);
  });

  it('checks single chords only in the overlays that own them', () => {
    expect(offenders(checksChords)).toEqual(CHORD_CHECKS);
  });

  it('listens for keys on the window only where the registry cannot', () => {
    expect(offenders(listensForKeys)).toEqual(WINDOW_KEY_LISTENERS);
  });

  it.each([
    ['a bind lookup', dispatchesBinds, 'const bind = matchKeybind(e, contexts);', true],
    ['a chord check as a lookup', dispatchesBinds, "if (isKeybind(e, 'maskDelete')) x();", false],
    ['a chord check', checksChords, "if (isKeybind(e, 'loupeClose')) close();", true],
    ['a window keydown', listensForKeys, '<svelte:window onkeydown={onKey} />', true],
    ['a window keyup', listensForKeys, '<svelte:window onblur={clear} onkeyup={track} />', true],
    ['a keydown listener', listensForKeys, "window.addEventListener('keydown', onKey);", true],
    ['an element keydown', listensForKeys, '<div role="grid" onkeydown={onKey}></div>', false]
  ])('classifies %s', (_, matches, source, flagged) => {
    expect(matches(source)).toBe(flagged);
  });
});
