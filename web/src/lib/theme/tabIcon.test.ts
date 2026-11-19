import { beforeEach, describe, expect, it, vi } from 'vitest';
import { faviconDataUrl } from './favicon';

const document = vi.hoisted(() => ({ writeFavicon: vi.fn<(href: string) => void>() }));

vi.mock('./document', () => document);

async function freshIcon(): Promise<typeof import('./tabIcon')> {
  vi.resetModules();
  return import('./tabIcon');
}

describe('tab icon', () => {
  beforeEach(() => document.writeFavicon.mockClear());

  it('waits for the theme accent before writing', async () => {
    const icon = await freshIcon();
    icon.showFaviconBadge({ kind: 'alert' });
    expect(document.writeFavicon).not.toHaveBeenCalled();
    icon.showFaviconAccent('#123456');
    expect(document.writeFavicon).toHaveBeenCalledWith(
      faviconDataUrl('#123456', { kind: 'alert' })
    );
  });

  it('writes each change once and drops the badge back to the theme icon', async () => {
    const icon = await freshIcon();
    icon.showFaviconAccent('#123456');
    icon.showFaviconBadge({ kind: 'progress', fraction: 0.4 });
    icon.showFaviconBadge({ kind: 'progress', fraction: 0.4 });
    icon.showFaviconBadge(null);
    expect(document.writeFavicon.mock.calls.map(([href]) => href)).toEqual([
      faviconDataUrl('#123456'),
      faviconDataUrl('#123456', { kind: 'progress', fraction: 0.4 }),
      faviconDataUrl('#123456')
    ]);
  });
});
