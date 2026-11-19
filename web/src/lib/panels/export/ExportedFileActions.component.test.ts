import { baseOptions, defaultExportForm } from '#lib/panels/export/settings.ts';
import type { ExportedFile } from '#lib/stores/editor/export.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { neutralEdits } from '#lib/types/edits.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ExportedFileActions from './ExportedFileActions.svelte';

type ActionProps = ComponentProps<typeof ExportedFileActions>;

const Host: Component<{ component: Component<ActionProps>; props: ActionProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function file(format: 'jpeg' | 'heic' = 'jpeg'): ExportedFile {
  return {
    blob: new Blob(['x'], { type: `image/${format}` }),
    name: `IMG_0001_edit.${format === 'jpeg' ? 'jpg' : format}`,
    options: { ...baseOptions(defaultExportForm()), format },
    edits: neutralEdits(),
    size: null,
    renderedAt: new Date(),
    renderMs: 1200,
    saved: false,
    warnings: []
  };
}

function render(props: Partial<ActionProps> = {}): void {
  component = mount(Host, {
    target,
    props: {
      component: ExportedFileActions,
      props: { file: file(), uploading: false, onSave: vi.fn(), onUpload: vi.fn(), ...props }
    }
  });
  flushSync();
}

function names(): string[] {
  return [...target.querySelectorAll('button')].map(
    (b) => b.getAttribute('aria-label') ?? b.textContent?.trim() ?? ''
  );
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  vi.unstubAllGlobals();
});

describe('ExportedFileActions', () => {
  it('offers Share only when the browser can share the file', () => {
    render();
    expect(names()).toEqual(['Save', 'Upload to Immich']);
    if (component) void unmount(component);

    const share = vi.fn(() => Promise.resolve());
    vi.stubGlobal('navigator', { ...navigator, canShare: () => true, share });
    render();
    expect(names()).toEqual(['Save', 'Share', 'Upload to Immich']);
    [...target.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Share')?.click();
    const [data] = share.mock.calls[0] as unknown as [{ files: File[] }];
    expect(data.files[0]?.name).toBe('IMG_0001_edit.jpg');
  });

  it('leads with Share in the compact touch row', () => {
    vi.stubGlobal('navigator', { ...navigator, canShare: () => true, share: vi.fn() });
    render({ compact: true });
    expect(names()).toEqual(['Share', 'Save', 'Upload to Immich']);
  });

  it('says why a HEIC file offers Save only', () => {
    vi.stubGlobal('navigator', { ...navigator, canShare: () => false, share: vi.fn() });
    render({ file: file('heic'), compact: true });
    expect(names()).toEqual(['Save', 'Upload to Immich']);
    expect(target.textContent).toContain("This browser can't share HEIC files.");
  });
});
