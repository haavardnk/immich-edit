import { browseView } from '#lib/stores/browseView.svelte.ts';
import { layoutState, resetLayout } from '#lib/testing/layoutMock.ts';
import { controls, layout, renderLayout, settle } from '#lib/testing/layoutParity.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ViewSheet from './ViewSheet.svelte';

vi.mock('$app/navigation', () => ({ afterNavigate: (): void => {}, goto: vi.fn() }));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  resetLayout({ mode: 'touch', phone: true });
  browseView.setCompact(true);
  browseView.setViewOpen(true);
});

afterEach(() => {
  browseView.setViewOpen(false);
  browseView.setCompact(false);
  resetLayout();
  vi.unstubAllGlobals();
});

function names(): string[] {
  return controls(document.body).map((control) => control.name);
}

describe('ViewSheet', () => {
  it('shows sort, size and the thumbnail info controls inline without filters', async () => {
    const destroy = renderLayout(layout('sheet', ViewSheet, { sortBasis: 'capture' }));
    try {
      await settle();
      const sheet = document.querySelector('[role="dialog"]');
      expect(
        document.getElementById(sheet?.getAttribute('aria-labelledby') ?? '')?.textContent?.trim()
      ).toBe('View');
      const groups = [...(sheet?.querySelectorAll('[role="radiogroup"]') ?? [])].map((group) =>
        group.getAttribute('aria-label')
      );
      expect(groups).toEqual(['Sort by', 'Order', 'Thumbnail size', 'Thumbnail info']);
      expect(names()).toEqual(
        expect.arrayContaining(['Close view', 'Reset thumbnail info', 'Edited', 'Lens'])
      );
      expect(names()).not.toContain('Reset all filters');
      expect(names().some((name) => name.startsWith('Show '))).toBe(false);
    } finally {
      await destroy();
    }
  });

  it('keeps only the size choice where the page has no sort', async () => {
    const destroy = renderLayout(layout('sheet', ViewSheet, { sortBasis: null }));
    try {
      await settle();
      const groups = [...document.querySelectorAll('[role="dialog"] [role="radiogroup"]')].map(
        (group) => group.getAttribute('aria-label')
      );
      expect(groups).toEqual(['Thumbnail size', 'Thumbnail info']);
    } finally {
      await destroy();
    }
  });

  it('closes from a Done button in portrait', async () => {
    const destroy = renderLayout(layout('sheet', ViewSheet, { sortBasis: 'capture' }));
    try {
      await settle();
      const done = [...document.querySelectorAll<HTMLButtonElement>('[role="dialog"] button')].find(
        (button) => button.textContent?.trim() === 'Done'
      );
      expect(done).toBeDefined();
      done?.click();
      await settle();
      expect(browseView.viewOpen).toBe(false);
    } finally {
      await destroy();
    }
  });

  it('drops the Done button in landscape', async () => {
    layoutState.landscape = true;
    const destroy = renderLayout(layout('sheet', ViewSheet, { sortBasis: 'capture' }));
    try {
      await settle();
      expect(names()).toContain('Close view');
      expect(names()).not.toContain('Done');
    } finally {
      await destroy();
    }
  });
});
