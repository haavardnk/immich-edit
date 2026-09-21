import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { renderShortcutsDoc } from './shortcutsDoc';

const DOC_PATH = fileURLToPath(new URL('../../../docs/shortcuts.md', import.meta.url));

describe('shortcut documentation', () => {
  it('matches the keybind registry', () => {
    const expected = renderShortcutsDoc();
    if (process.env.BAKE_SHORTCUTS_DOC === '1') writeFileSync(DOC_PATH, expected);
    expect(readFileSync(DOC_PATH, 'utf8')).toBe(expected);
  });
});
