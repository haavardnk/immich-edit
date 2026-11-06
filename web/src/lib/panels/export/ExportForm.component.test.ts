import BulkExport from '#lib/components/browse/bulk/BulkExport.svelte';
import Export from '#lib/panels/Export.svelte';
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
  type ControlRef,
  type Layout
} from '#lib/testing/layoutParity.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ExportForm from './ExportForm.svelte';
import ProofMismatchNotice from './ProofMismatchNotice.svelte';

vi.mock('$app/navigation', () => ({ afterNavigate: (): void => {}, goto: vi.fn() }));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

const button = (name: string): ControlRef => ({ role: 'button', name });

function asset(id: string, ownerId: string): AssetSummary {
  return {
    id,
    ownerId,
    originalFileName: `${id}.ARW`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

function layouts(
  destination: 'download' | 'immich'
): Layout<'body' | 'zip body' | 'editor panel' | 'bulk export'>[] {
  const immich = destination === 'immich';
  return [
    layout('body', ExportForm, { scope: exportScope({ assets: [], count: 1, trashOn: false }) }),
    layout('zip body', ExportForm, {
      scope: exportScope({ assets: [], count: 0, trashOn: false }),
      downloadLabel: 'Download ZIP'
    }),
    layout(
      'editor panel',
      Export,
      {},
      { exclude: [button(immich ? 'Upload JPEG to Immich' : 'Export JPEG')] }
    ),
    layout(
      'bulk export',
      BulkExport,
      {},
      { exclude: [button(immich ? 'Export 0 to Immich' : 'Download 0 as JPEG ZIP')] }
    )
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
      expect(found['editor panel']).toEqual(found.body);
      expect(found['bulk export']).toEqual(found['zip body']);
    }
  );

  it('adds the soft-proof notice body to the editor panel only', async () => {
    editor.proofSpace = 'displayp3';
    const found = await layoutControls([
      layout('form', ExportForm, { scope: exportScope({ assets: [], count: 1, trashOn: false }) }),
      layout('notice', ProofMismatchNotice, {}),
      layout('editor panel', Export, {}, { exclude: [button('Export JPEG')] }),
      layout('bulk export', BulkExport, {}, { exclude: [button('Download 0 as JPEG ZIP')] })
    ]);
    expect(found.notice).toEqual([{ role: 'button', name: 'Export Display P3' }]);
    expect(found['editor panel']).toEqual(mergeControls(found.form, found.notice));
    expect(found['bulk export']).not.toContainEqual(found.notice[0]);
  });

  it.each([
    ['mixed ownership', ['me', 'other']],
    ['no owned photo', ['other', 'other']]
  ])('matches the bulk Immich options for %s', async (_, owners) => {
    const assets = owners.map((owner, i) => asset(`a${i}`, owner));
    browsing.set(assets);
    selection.replace(assets.map((a) => a.id));
    immichCapabilities.trash = true;
    exportSettings.destination = 'immich';
    const found = await layoutControls([
      layout('body', ExportForm, {
        scope: exportScope({ assets, count: assets.length, trashOn: true }),
        downloadLabel: 'Download ZIP'
      }),
      layout('bulk export', BulkExport, {}, { exclude: [button('Export 2 to Immich')] })
    ]);
    const replace = found.body.find((control) => control.name === 'Replace original');
    expect(replace?.disabled).toBe(owners.includes('me') ? undefined : true);
    expect(found['bulk export']).toEqual(found.body);
  });
});
