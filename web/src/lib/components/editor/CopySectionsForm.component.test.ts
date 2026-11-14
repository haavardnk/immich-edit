import CopyDialog from '#lib/components/shell/CopyDialog.svelte';
import { DEFAULT_COPY_SELECTION, selectionOf } from '#lib/editor/copyTools.ts';
import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
import {
  layout,
  layoutControls,
  mergeControls,
  type ControlRef
} from '#lib/testing/layoutParity.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import CopySectionsActions from './CopySectionsActions.svelte';
import CopySectionsForm from './CopySectionsForm.svelte';

const button = (name: string): ControlRef => ({ role: 'button', name });

beforeEach(() => {
  copyDialog.show(neutralEdits(), undefined, 'IMG_4120.ARW');
});

afterEach(() => {
  copyDialog.cancel();
  copyDialog.selection = { ...DEFAULT_COPY_SELECTION };
});

describe('CopySectionsForm layout parity', () => {
  it('exposes the same controls standalone and in the copy dialog', async () => {
    const found = await layoutControls([
      layout('body', CopySectionsForm, {}),
      layout(
        'dialog',
        CopyDialog,
        {},
        {
          exclude: [
            button('Check all'),
            button('Check none'),
            button('Cancel'),
            button('Copy'),
            button('close')
          ]
        }
      )
    ]);
    expect(found.body).toHaveLength(24);
    expect(found.dialog).toEqual(found.body);
  });

  it('exposes the form and its actions together in the copy dialog', async () => {
    const found = await layoutControls([
      layout('form', CopySectionsForm, {}),
      layout('actions', CopySectionsActions, {}),
      layout(
        'dialog',
        CopyDialog,
        {},
        { exclude: [button('Cancel'), button('Copy'), button('close')] }
      )
    ]);
    expect(found.actions).toEqual([button('Check all'), button('Check none')]);
    expect(found.dialog).toEqual(mergeControls(found.form, found.actions));
  });

  it('lists only the copied tools when pasting', async () => {
    copyDialog.showPaste(
      { edits: neutralEdits(), tools: selectionOf(['hsl', 'vignette']), touched: null },
      () => undefined
    );
    const found = await layoutControls([layout('form', CopySectionsForm, {})]);
    expect(found.form.filter((control) => control.role === 'checkbox').map((c) => c.name)).toEqual([
      'Develop',
      'Effects',
      'HSL',
      'Vignette'
    ]);
  });
});
