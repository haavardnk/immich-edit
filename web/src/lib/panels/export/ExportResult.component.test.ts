import Export from '#lib/panels/Export.svelte';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import { exportScope } from '#lib/panels/export/scope.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { layout, layoutControls, mergeControls } from '#lib/testing/layoutParity.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ExportForm from './ExportForm.svelte';
import ExportResult from './ExportResult.svelte';

vi.mock('$app/navigation', () => ({ afterNavigate: (): void => {}, goto: vi.fn() }));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

const failed = {
  kind: 'error' as const,
  message: 'Export failed',
  warnings: ['Lens profile missing']
};

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  exportSettings.destination = 'download';
});

afterEach(() => {
  editor.lastDownload = null;
  vi.unstubAllGlobals();
});

describe('ExportResult layout parity', () => {
  it('exposes the same controls standalone and in the export panel', async () => {
    editor.lastDownload = failed;
    const found = await layoutControls([
      layout('result', ExportResult, { result: failed, busy: false, onRetry: (): void => {} }),
      layout('form', ExportForm, { scope: exportScope({ assets: [], count: 1, trashOn: false }) }),
      layout('panel', Export, {}, { exclude: [{ role: 'button', name: 'Export JPEG' }] })
    ]);

    expect(found.result).toEqual([{ role: 'button', name: 'Retry' }]);
    expect(found.panel).toEqual(mergeControls(found.form, found.result));
  });
});
