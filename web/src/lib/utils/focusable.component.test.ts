import { firstFocusable, focusables } from '#lib/utils/focusable.ts';
import { afterEach, describe, expect, it } from 'vitest';

afterEach(() => {
  document.body.innerHTML = '';
});

describe('focusable', () => {
  it('finds links, buttons, fields and tabindex elements in document order', () => {
    document.body.innerHTML = `
      <a id="plain">No href</a>
      <a id="link" href="/a">A</a>
      <button id="button">B</button>
      <input id="input" />
      <select id="select"></select>
      <textarea id="textarea"></textarea>
      <div id="custom" tabindex="0">C</div>
    `;
    expect(focusables(document.body).map((element) => element.id)).toEqual([
      'link',
      'button',
      'input',
      'select',
      'textarea',
      'custom'
    ]);
  });

  it('skips disabled, unfocusable and inert elements for the first focusable', () => {
    document.body.innerHTML = `
      <button disabled>Disabled</button>
      <span tabindex="-1">Skipped</span>
      <div inert><a href="/inert">Inert</a></div>
      <a id="first" href="/first">First</a>
      <button>Later</button>
    `;
    expect(firstFocusable(document.body)?.id).toBe('first');
  });

  it('returns null when nothing can take focus', () => {
    document.body.innerHTML = '<button disabled>Disabled</button><span tabindex="-1">No</span>';
    expect(firstFocusable(document.body)).toBeNull();
  });
});
