import { afterEach, describe, expect, it } from 'vitest';
import { controls, mergeControls, withoutControls } from './layoutParity';

let root: HTMLDivElement;

afterEach(() => root.remove());

function fixture(html: string): HTMLDivElement {
  root = document.createElement('div');
  root.innerHTML = html;
  document.body.appendChild(root);
  return root;
}

describe('controls', () => {
  it('lists implicit and explicit roles with their accessible names, sorted', () => {
    const el = fixture(`
      <button>  Save   copy </button>
      <button aria-label="Close"><svg></svg></button>
      <a href="/x" title="Open">  </a>
      <a>plain anchor</a>
      <label for="q">Query</label><input id="q" />
      <label><input type="checkbox" /> Favourites</label>
      <input type="range" aria-label="Amount" />
      <input type="hidden" value="x" />
      <span id="t">Title text</span><div role="switch" aria-labelledby="t"></div>
      <div role="group" aria-label="Ignored"></div>
      <textarea placeholder="Notes"></textarea>
      <select aria-label="Format"><option>JPEG</option></select>
    `);
    expect(controls(el)).toEqual([
      { role: 'button', name: 'Close' },
      { role: 'button', name: 'Save copy' },
      { role: 'checkbox', name: 'Favourites', checked: false },
      { role: 'combobox', name: 'Format' },
      { role: 'link', name: 'Open' },
      { role: 'slider', name: 'Amount' },
      { role: 'switch', name: 'Title text' },
      { role: 'textbox', name: 'Notes' },
      { role: 'textbox', name: 'Query' }
    ]);
  });

  it('records disabled, checked, pressed and expanded state', () => {
    const el = fixture(`
      <button disabled>Off</button>
      <button aria-disabled="true" aria-pressed="true">Bold</button>
      <button aria-expanded="false">Menu</button>
      <input type="checkbox" aria-label="Done" checked />
      <div role="checkbox" aria-label="Some" aria-checked="mixed"></div>
      <fieldset disabled><input aria-label="Locked" /></fieldset>
    `);
    expect(controls(el)).toEqual([
      { role: 'button', name: 'Bold', disabled: true, pressed: true },
      { role: 'button', name: 'Menu', expanded: false },
      { role: 'button', name: 'Off', disabled: true },
      { role: 'checkbox', name: 'Done', checked: true },
      { role: 'checkbox', name: 'Some', checked: 'mixed' },
      { role: 'textbox', name: 'Locked', disabled: true }
    ]);
  });

  it('skips hidden and inert subtrees', () => {
    const el = fixture(`
      <div aria-hidden="true"><button>Hidden</button></div>
      <div inert><button>Inert</button></div>
      <div hidden><button>Gone</button></div>
      <button>Shown</button>
    `);
    expect(controls(el)).toEqual([{ role: 'button', name: 'Shown' }]);
  });
});

describe('mergeControls', () => {
  it('concatenates and sorts by role, name and state', () => {
    expect(
      mergeControls(
        [{ role: 'radio', name: 'B' }],
        [
          { role: 'button', name: 'A', disabled: true },
          { role: 'button', name: 'A' }
        ]
      )
    ).toEqual([
      { role: 'button', name: 'A' },
      { role: 'button', name: 'A', disabled: true },
      { role: 'radio', name: 'B' }
    ]);
  });
});

describe('withoutControls', () => {
  it('matches excluded controls by role and name only', () => {
    expect(
      withoutControls(
        [{ role: 'button', name: 'A', disabled: true }],
        [{ role: 'button', name: 'A' }]
      )
    ).toEqual([]);
  });

  it('removes one occurrence per excluded control', () => {
    const list = [
      { role: 'button', name: 'A' },
      { role: 'button', name: 'A' },
      { role: 'radio', name: 'B' }
    ];
    expect(withoutControls(list, [{ role: 'button', name: 'A' }])).toEqual([
      { role: 'button', name: 'A' },
      { role: 'radio', name: 'B' }
    ]);
  });

  it('fails when an excluded control is missing', () => {
    expect(() => withoutControls([], [{ role: 'button', name: 'A' }])).toThrow(
      'Missing button "A"'
    );
  });
});
