import { describe, expect, it } from 'vitest';
import { editorHref, validReturnPath } from './editorNavigation';

describe('editor navigation context', () => {
  it('preserves route query state in the editor URL', () => {
    const href = editorHref('asset 1', '/albums/one?sort=desc&filter=raw');

    expect(href).toBe('/assets/asset%201?from=%2Falbums%2Fone%3Fsort%3Ddesc%26filter%3Draw');
  });

  it('rejects external and nested editor return paths', () => {
    expect(validReturnPath('//example.com')).toBe(false);
    expect(validReturnPath('/assets/other')).toBe(false);
    expect(editorHref('asset', '//example.com')).toBe('/assets/asset');
  });
});
