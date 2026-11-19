import { focusFirst, focusTargetsAround } from '#lib/utils/focusAround.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';

function region(): HTMLElement {
  const element = document.getElementById('region');
  if (!element) throw new Error('no region');
  return element;
}

function ids(elements: readonly HTMLElement[]): string[] {
  return elements.map((element) => element.id);
}

function rendered(hidden: readonly string[]): void {
  vi.spyOn(Element.prototype, 'getClientRects').mockImplementation(function (this: Element) {
    return (hidden.includes(this.id) ? [] : [new DOMRect()]) as unknown as DOMRectList;
  });
}

afterEach(() => {
  vi.restoreAllMocks();
  Reflect.deleteProperty(Element.prototype, 'checkVisibility');
  document.body.innerHTML = '';
});

describe('focusTargetsAround', () => {
  it('lists the targets before the region nearest first, then the ones after', () => {
    document.body.innerHTML = `
      <a id="far" href="/far">Far</a>
      <nav><button id="near">Near</button></nav>
      <div id="wrapper" tabindex="0">
        <ul id="region"><li><a id="inside" href="/inside">Inside</a></li></ul>
      </div>
      <input id="next" />
      <textarea id="last"></textarea>
    `;
    expect(ids(focusTargetsAround(region()))).toEqual(['near', 'far', 'next', 'last']);
  });
});

describe('focusFirst', () => {
  const targets = `
    <button id="disabled" disabled>Disabled</button>
    <span id="skipped" tabindex="-1">Skipped</span>
    <div inert><button id="inert">Inert</button></div>
    <button id="hidden">Hidden</button>
    <button id="shown">Shown</button>
  `;

  function all(): HTMLElement[] {
    return [...document.body.querySelectorAll<HTMLElement>('[id]')];
  }

  it('falls back to client rects without checkVisibility', () => {
    document.body.innerHTML = targets;
    rendered(['hidden']);
    focusFirst(all());
    expect(document.activeElement?.id).toBe('shown');
  });

  it('uses checkVisibility when the browser has it', () => {
    document.body.innerHTML = targets;
    rendered([]);
    Object.defineProperty(Element.prototype, 'checkVisibility', {
      configurable: true,
      value(this: Element) {
        return this.id !== 'hidden';
      }
    });
    focusFirst(all());
    expect(document.activeElement?.id).toBe('shown');
  });

  it('leaves focus alone when nothing can take it', () => {
    document.body.innerHTML = targets;
    rendered(['hidden', 'shown']);
    const before = document.activeElement;
    focusFirst(all());
    expect(document.activeElement).toBe(before);
  });
});
