import { layout, renderLayout, settle } from '#lib/testing/layoutParity.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';
import SaveViewForm from './SaveViewForm.svelte';

let cleanup: (() => Promise<void>) | null = null;

async function render(initial: string) {
  const onSubmit = vi.fn().mockResolvedValue(undefined);
  const onCancel = vi.fn();
  cleanup = renderLayout(
    layout('form', SaveViewForm, { initial, label: 'View name', onSubmit, onCancel })
  );
  await new Promise((resolve) => requestAnimationFrame(resolve));
  await settle();
  const input = document.querySelector<HTMLInputElement>('input[aria-label="View name"]');
  const form = document.querySelector<HTMLFormElement>('form[aria-label="View name"]');
  const save = [...document.querySelectorAll('button')].find(
    (b) => b.textContent.trim() === 'Save'
  );
  if (!input || !form || !save) throw new Error('form missing');
  return { input, form, save, onSubmit, onCancel };
}

async function type(input: HTMLInputElement, value: string): Promise<void> {
  input.value = value;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  await settle();
}

afterEach(async () => {
  await cleanup?.();
  cleanup = null;
});

describe('SaveViewForm', () => {
  it('prefills the name, focused and selected', async () => {
    const { input } = await render('Norway · X-T5');
    expect(input.value).toBe('Norway · X-T5');
    expect(document.activeElement).toBe(input);
    expect(input.selectionStart).toBe(0);
    expect(input.selectionEnd).toBe('Norway · X-T5'.length);
  });

  it('submits the trimmed name', async () => {
    const { input, form, onSubmit } = await render('Photos');
    await type(input, '  Keepers  ');
    form.requestSubmit();
    await settle();
    expect(onSubmit).toHaveBeenCalledWith('Keepers');
  });

  it('cancels on Escape without reaching window shortcuts', async () => {
    const { input, onCancel } = await render('Photos');
    const reached = vi.fn();
    window.addEventListener('keydown', reached);
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    window.removeEventListener('keydown', reached);
    expect(onCancel).toHaveBeenCalledOnce();
    expect(reached).not.toHaveBeenCalled();
  });

  it('allows 80 characters and refuses blank or longer names', async () => {
    const { input, form, save, onSubmit } = await render('Photos');
    await type(input, 'x'.repeat(80));
    expect(save.disabled).toBe(false);
    expect(input.hasAttribute('aria-describedby')).toBe(false);
    await type(input, 'x'.repeat(81));
    expect(save.disabled).toBe(true);
    expect(document.body.textContent).toContain('Up to 80 characters');
    form.requestSubmit();
    await type(input, '   ');
    expect(save.disabled).toBe(true);
    const message = document.getElementById(input.getAttribute('aria-describedby') ?? '');
    expect(message?.textContent).toBe('Enter a name');
    form.requestSubmit();
    await settle();
    expect(onSubmit).not.toHaveBeenCalled();
  });
});
