import { describe, expect, it, vi } from 'vitest';
import { currentPath, isEditorPath, onEditorRoute, returnPath } from './route';

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/assets/a?from=%2Falbums%2Fa1%3Fsort%3Dasc') }
}));

describe('route', () => {
  it.each([
    ['/assets/a', true],
    ['/assets', false],
    ['/albums/a1', false]
  ])('treats %s as an editor path: %s', (path, editor) => {
    expect(isEditorPath(path)).toBe(editor);
  });

  it('reads the current page', () => {
    expect(onEditorRoute()).toBe(true);
    expect(currentPath()).toBe('/assets/a?from=%2Falbums%2Fa1%3Fsort%3Dasc');
    expect(returnPath()).toBe('/albums/a1?sort=asc');
  });
});
