import { DEFAULT_COPY_SECTIONS } from '#lib/editor/copyPaste.ts';
import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
import {
  layout,
  layoutControls,
  mergeControls,
  type ControlRef
} from '#lib/testing/layoutParity.ts';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import CopyDialog from '#lib/components/shell/CopyDialog.svelte';
import CopySectionsActions from './CopySectionsActions.svelte';
import CopySectionsForm from './CopySectionsForm.svelte';

const button = (name: string): ControlRef => ({ role: 'button', name });

beforeEach(() => {
  copyDialog.open = true;
});

afterEach(() => {
  copyDialog.cancel();
  copyDialog.sections = { ...DEFAULT_COPY_SECTIONS };
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
    expect(found.body).toHaveLength(10);
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
});
