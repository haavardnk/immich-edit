import BulkExport from '#lib/components/browse/bulk/BulkExport.svelte';
import Export from '#lib/panels/Export.svelte';
import { BulkExporter } from '#lib/panels/export/bulkExport.svelte.ts';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import { exportScope } from '#lib/panels/export/scope.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import {
  layout,
  layoutControls,
  mergeControls,
  settle,
  type ControlRef,
  type Layout
} from '#lib/testing/layoutParity.ts';
import { asset } from '#lib/testing/assets.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ExportForm from './ExportForm.svelte';
import { EarlierExports } from './earlierExports.svelte';
import ExportPresetMenu from './ExportPresetMenu.svelte';
import { PresetSelection } from './presetSelection.svelte';
import ProofMismatchNotice from './ProofMismatchNotice.svelte';

vi.mock('$app/navigation', () => ({ afterNavigate: (): void => {}, goto: vi.fn() }));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

const button = (name: string): ControlRef => ({ role: 'button', name });
const saveAutomatically: ControlRef = { role: 'checkbox', name: 'Save automatically' };

const presetMenu = (): Layout<'preset menu'> =>
  layout('preset menu', ExportPresetMenu, {
    exporter: {
      presets: new PresetSelection(),
      exportingPresets: false,
      earlierPresets: new EarlierExports(),
      presetQuery: null,
      scope: exportScope({ assets: [], count: 1, trashOn: false }),
      exportPresets: async () => false
    }
  });

async function unfold(root: HTMLElement): Promise<void> {
  for (const trigger of root.querySelectorAll<HTMLElement>(
    '[data-collapsible-trigger][aria-expanded="false"]'
  )) {
    trigger.click();
  }
  await settle();
}

const unfolded = { open: unfold };

function layouts(
  destination: 'download' | 'immich'
): Layout<'body' | 'zip body' | 'editor panel' | 'bulk export' | 'preset menu'>[] {
  const immich = destination === 'immich';
  return [
    presetMenu(),
    layout(
      'body',
      ExportForm,
      { scope: exportScope({ assets: [], count: 1, trashOn: false }) },
      unfolded
    ),
    layout(
      'zip body',
      ExportForm,
      { scope: exportScope({ assets: [], count: 0, trashOn: false }) },
      unfolded
    ),
    layout(
      'editor panel',
      Export,
      {},
      {
        ...unfolded,
        exclude: immich
          ? [button('Upload JPEG to Immich')]
          : [button('Export JPEG'), saveAutomatically]
      }
    ),
    layout('bulk export', BulkExport, { exporter: new BulkExporter(() => {}) }, unfolded)
  ];
}

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
});

afterEach(() => {
  exportSettings.destination = 'download';
  editor.proofSpace = 'srgb';
  immichCapabilities.trash = false;
  selection.clear();
  browsing.set([]);
  session.clear();
  vi.unstubAllGlobals();
});

describe('ExportForm layout parity', () => {
  it.each(['download', 'immich'] as const)(
    'exposes the same controls in every container (%s)',
    async (destination) => {
      exportSettings.destination = destination;
      const found = await layoutControls(layouts(destination));
      expect(found.body.length).toBeGreaterThan(10);
      expect(found['preset menu']).toEqual([{ ...button('Export presets'), expanded: false }]);
      expect(found['editor panel']).toEqual(mergeControls(found.body, found['preset menu']));
      expect(found['bulk export']).toEqual(found['zip body']);
    }
  );

  it('adds the soft-proof notice body to the editor panel only', async () => {
    editor.proofSpace = 'displayp3';
    const found = await layoutControls([
      presetMenu(),
      layout(
        'form',
        ExportForm,
        { scope: exportScope({ assets: [], count: 1, trashOn: false }) },
        unfolded
      ),
      layout('notice', ProofMismatchNotice, {}),
      layout(
        'editor panel',
        Export,
        {},
        { ...unfolded, exclude: [button('Export JPEG'), saveAutomatically] }
      ),
      layout('bulk export', BulkExport, { exporter: new BulkExporter(() => {}) }, unfolded)
    ]);
    expect(found.notice).toEqual([{ role: 'button', name: 'Export Display P3' }]);
    expect(found['editor panel']).toEqual(
      mergeControls(found.form, found.notice, found['preset menu'])
    );
    expect(found['bulk export']).not.toContainEqual(found.notice[0]);
  });

  it.each([
    ['mixed ownership', ['me', 'other']],
    ['no owned photo', ['other', 'other']]
  ])('matches the bulk Immich options for %s', async (_, owners) => {
    const assets = owners.map((owner, i) => asset(`a${i}`, { ownerId: owner }));
    browsing.set(assets);
    selection.replace(assets.map((a) => a.id));
    immichCapabilities.trash = true;
    exportSettings.destination = 'immich';
    const found = await layoutControls([
      presetMenu(),
      layout(
        'body',
        ExportForm,
        { scope: exportScope({ assets, count: assets.length, trashOn: true }) },
        unfolded
      ),
      layout('bulk export', BulkExport, { exporter: new BulkExporter(() => {}) }, unfolded)
    ]);
    const replace = found.body.find((control) => control.name === 'Replace original');
    expect(replace?.disabled).toBe(owners.includes('me') ? undefined : true);
    expect(found['bulk export']).toEqual(found.body);
  });
});
