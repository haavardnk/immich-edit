import type { CullActions, CullState } from '#lib/browse/cullState.ts';
import type { TagActions } from '#lib/browse/loupeActions.ts';
import LoupeCullBar from '#lib/components/browse/LoupeCullBar.svelte';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { resetLayout } from '#lib/testing/layoutMock.ts';
import type { TagRef } from '#lib/types/asset.ts';
import {
  createRawSnippet,
  flushSync,
  mount,
  unmount,
  type Component,
  type ComponentProps
} from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

vi.mock('#lib/stores/library.svelte.ts', () => ({
  library: { tags: [], ensureTags: () => Promise.resolve() }
}));

type BarProps = ComponentProps<typeof LoupeCullBar>;

const Host: Component<{ component: Component<BarProps>; props: BarProps }> = ComponentHost;

const OWNED: CullState = {
  rating: 2,
  isFavorite: true,
  rejected: false,
  label: 'green',
  owned: true
};

const TAGS: TagRef[] = [
  { id: 'tag-sky', name: 'Sky', value: 'Sky' },
  { id: 'tag-sea', name: 'Sea', value: 'Places/Sea' },
  { id: 'tag-reject', name: 'reject', value: 'immich-edit/reject' }
];

const trailing = createRawSnippet(() => ({
  render: () => '<button type="button" data-testid="trailing">Edit</button>'
}));

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function actions(): CullActions {
  return { rate: vi.fn(), favorite: vi.fn(), reject: vi.fn(), label: vi.fn() };
}

function tagActions(): TagActions {
  return {
    add: vi.fn(() => Promise.resolve()),
    remove: vi.fn(() => Promise.resolve()),
    create: vi.fn(() => Promise.resolve(null))
  };
}

function render(props: Partial<BarProps> & Pick<BarProps, 'density'>): BarProps {
  const full: BarProps = {
    cull: OWNED,
    actions: actions(),
    tags: TAGS,
    tagActions: tagActions(),
    fileName: 'IMG_4120.ARW',
    trailing,
    ...props
  };
  component = mount(Host, { target, props: { component: LoupeCullBar, props: full } });
  flushSync();
  return full;
}

function bar(): HTMLElement {
  const element = target.querySelector<HTMLElement>('nav[aria-label="Photo actions"]');
  if (!element) throw new Error('no cull bar');
  return element;
}

function text(element: Element): string {
  return element.textContent.replace(/\s+/g, ' ').trim();
}

function button(name: string): HTMLButtonElement {
  const found = [...target.querySelectorAll<HTMLButtonElement>('button')].find(
    (element) => (element.getAttribute('aria-label') ?? text(element)) === name
  );
  if (!found) throw new Error(`no button ${name}`);
  return found;
}

beforeEach(() => {
  resetLayout({ mode: 'touch' });
  browseView.loupeTagsOpen = false;
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  resetLayout();
});

describe('LoupeCullBar', () => {
  it('stacks the stars above labelled actions and the trailing control', () => {
    const props = render({ density: 'stacked' });
    expect(bar().dataset.density).toBe('stacked');
    const [stars, actionRow, trailingCell] = [...bar().children];
    expect(stars?.querySelector('[role="radiogroup"]')).not.toBeNull();
    expect(stars?.className).toContain('col-span-2');
    expect([...(actionRow?.querySelectorAll('button') ?? [])].map(text)).toEqual([
      'Unfavorite',
      'Reject',
      'Label',
      '2 Tags (2)'
    ]);
    expect(trailingCell?.querySelector('[data-testid="trailing"]')).not.toBeNull();

    button('Unfavorite').click();
    button('Reject').click();
    button('3 stars').click();
    expect(props.actions.favorite).toHaveBeenCalledOnce();
    expect(props.actions.reject).toHaveBeenCalledOnce();
    expect(props.actions.rate).toHaveBeenCalledWith(3);
    expect(button('Green label').dataset.label).toBe('green');
  });

  it('lays everything out in one row with fixed-width actions', () => {
    render({ density: 'row' });
    expect(bar().dataset.density).toBe('row');
    expect(bar().className).toContain('flex');
    expect(bar().className).not.toContain('grid');
    expect(button('Reject').className).toContain('w-16');
    expect(bar().lastElementChild?.className).toContain('ml-auto');
  });

  it('opens the tag picker from the Tags action', () => {
    render({ density: 'row' });
    button('2 Tags (2)').click();
    flushSync();
    expect(browseView.loupeTagsOpen).toBe(true);
  });

  it('locks every cull action on a photo the user does not own', () => {
    const props = render({ density: 'stacked', cull: { ...OWNED, owned: false } });
    for (const name of ['Unfavorite', 'Reject', 'Green label', '2 Tags (2)']) {
      expect(button(name).disabled).toBe(true);
    }
    button('3 stars').click();
    expect(props.actions.rate).not.toHaveBeenCalled();
    expect(target.querySelector('[data-testid="trailing"]')).not.toBeNull();
  });
});
