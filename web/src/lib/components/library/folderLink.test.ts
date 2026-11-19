import { describe, expect, it } from 'vitest';
import { folderLinkClass } from '#lib/components/library/folderLink.ts';

describe('folderLinkClass', () => {
  it('highlights only the active folder', () => {
    expect(folderLinkClass(true)).toContain('text-primary');
    expect(folderLinkClass(true)).not.toContain('hover:bg-subtle');
    expect(folderLinkClass(false)).toContain('hover:bg-subtle');
    expect(folderLinkClass(false)).not.toContain('bg-primary/15');
  });
});
