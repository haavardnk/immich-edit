import type { TagSummary } from '#lib/api/tags.ts';
import { LIST_FILTER_MIN } from '#lib/browse/listFilter.ts';
import { library } from '#lib/stores/library.svelte.ts';
import { layout, renderLayout, settle } from '#lib/testing/layoutParity.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';
import TagList from './TagList.svelte';

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

const NAMES = ['Iceland', 'Finland', 'landscape', 'Lantern', 'Ålesund', 'Øya'];

function tags(count: number): TagSummary[] {
  return Array.from({ length: count }, (_, index) => {
    const name = NAMES[index] ?? `Tag ${index}`;
    return { id: `t${index}`, name, value: name, createdAt: null };
  });
}

let cleanup: (() => Promise<void>) | null = null;

async function render(count: number): Promise<HTMLElement> {
  library.tags = tags(count);
  cleanup = renderLayout(layout('tags', TagList, {}));
  await settle();
  return document.body;
}

function field(root: HTMLElement): HTMLInputElement | null {
  return root.querySelector<HTMLInputElement>('input[aria-label="Filter tags"]');
}

function rowNames(root: HTMLElement): string[] {
  return [...root.querySelectorAll<HTMLAnchorElement>('a[href^="/tags/"]')].map(
    (link) => link.textContent?.replace(/\s+/g, ' ').trim() ?? ''
  );
}

async function type(input: HTMLInputElement, value: string): Promise<void> {
  input.value = value;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  await settle();
}

async function press(input: HTMLInputElement, key: string): Promise<boolean> {
  let reached = false;
  const listener = (): void => {
    reached = true;
  };
  window.addEventListener('keydown', listener);
  input.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
  window.removeEventListener('keydown', listener);
  await settle();
  return reached;
}

function status(root: HTMLElement): string {
  return root.querySelector('[role="status"]')?.textContent?.trim() ?? '';
}

afterEach(async () => {
  await cleanup?.();
  cleanup = null;
  library.tags = [];
});

describe('ListFilter', () => {
  it('hides the field for short lists', async () => {
    const root = await render(LIST_FILTER_MIN);
    expect(field(root)).toBeNull();
    expect(rowNames(root)).toHaveLength(LIST_FILTER_MIN);
  });

  it('filters by a folded substring and highlights the match', async () => {
    const root = await render(LIST_FILTER_MIN + 4);
    const input = field(root);
    if (!input) throw new Error('field missing');
    expect(status(root)).toBe('');
    await type(input, 'LAN');
    expect(rowNames(root)).toEqual(['Iceland', 'Finland', 'landscape', 'Lantern']);
    expect([...root.querySelectorAll('mark')].map((mark) => mark.textContent)).toEqual([
      'lan',
      'lan',
      'lan',
      'Lan'
    ]);
    expect(status(root)).toBe(`4 of ${LIST_FILTER_MIN + 4} tags`);
    await type(input, 'ales');
    expect(rowNames(root)).toEqual(['Ålesund']);
    await type(input, 'zzz');
    expect(rowNames(root)).toEqual([]);
    expect(status(root)).toBe('No matching tags');
  });

  it('clears on Escape, then blurs, without reaching window shortcuts', async () => {
    const root = await render(LIST_FILTER_MIN + 1);
    const input = field(root);
    if (!input) throw new Error('field missing');
    input.focus();
    await type(input, 'lan');
    expect(await press(input, 'Escape')).toBe(false);
    expect(input.value).toBe('');
    expect(rowNames(root)).toHaveLength(LIST_FILTER_MIN + 1);
    expect(document.activeElement).toBe(input);
    expect(await press(input, 'Escape')).toBe(false);
    expect(document.activeElement).not.toBe(input);
  });

  it('moves focus to the first row on ArrowDown', async () => {
    const root = await render(LIST_FILTER_MIN + 1);
    const input = field(root);
    if (!input) throw new Error('field missing');
    input.focus();
    await type(input, 'fin');
    await press(input, 'ArrowDown');
    expect(document.activeElement?.textContent?.trim()).toBe('Finland');
  });
});
