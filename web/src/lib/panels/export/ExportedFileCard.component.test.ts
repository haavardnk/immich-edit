import { baseOptions, defaultExportForm } from '#lib/panels/export/settings.ts';
import type { ExportedFile } from '#lib/stores/editor/export.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { neutralEdits } from '#lib/types/edits.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ExportedFileCard from './ExportedFileCard.svelte';

type CardProps = ComponentProps<typeof ExportedFileCard>;

const Host: Component<{ component: Component<CardProps>; props: CardProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function file(patch: Partial<ExportedFile> = {}): ExportedFile {
  return {
    blob: new Blob([new Uint8Array(4_200_000)], { type: 'image/jpeg' }),
    name: 'IMG_0001_edit.jpg',
    options: baseOptions(defaultExportForm()),
    edits: neutralEdits(),
    size: { w: 2048, h: 1365 },
    renderedAt: new Date(2026, 10, 14, 14, 32),
    saved: false,
    warnings: [],
    ...patch
  };
}

function render(props: CardProps): void {
  component = mount(Host, { target, props: { component: ExportedFileCard, props } });
  flushSync();
}

function button(name: string): HTMLButtonElement | null {
  return [...target.querySelectorAll('button')].find((b) => b.textContent?.trim() === name) ?? null;
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
  URL.createObjectURL = vi.fn(() => 'blob:thumb');
  URL.revokeObjectURL = vi.fn();
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
});

describe('ExportedFileCard', () => {
  it('describes the file and runs Save and Upload', () => {
    const onSave = vi.fn();
    const onUpload = vi.fn();
    render({
      file: file({ warnings: ['Metadata not copied'] }),
      uploading: false,
      onSave,
      onUpload
    });

    const text = target.textContent ?? '';
    expect(text).toContain('IMG_0001_edit.jpg');
    expect(text).toContain('4.2 MB · 2048 × 1365');
    expect(text).toContain('Ready, not saved yet');
    expect(text).toContain('Metadata not copied');
    expect(target.querySelector('img')?.getAttribute('src')).toBe('blob:thumb');

    button('Save')?.click();
    button('Upload to Immich')?.click();
    expect(onSave).toHaveBeenCalledOnce();
    expect(onUpload).toHaveBeenCalledOnce();
  });

  it('shows Saved, falls back to an icon and hides Copy without clipboard images', () => {
    render({ file: file({ saved: true }), uploading: false, onSave: vi.fn(), onUpload: vi.fn() });
    expect(target.textContent).toContain('Saved');
    expect(button('Copy image')).toBeNull();

    target.querySelector('img')?.dispatchEvent(new Event('error'));
    flushSync();
    expect(target.querySelector('img')).toBeNull();
    expect(target.querySelector('svg')).not.toBeNull();
  });

  it('revokes the thumbnail URL when it unmounts', () => {
    render({ file: file(), uploading: false, onSave: vi.fn(), onUpload: vi.fn() });
    if (component) void unmount(component);
    component = null;
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:thumb');
  });
});
