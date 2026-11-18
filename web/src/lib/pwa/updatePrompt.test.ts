import { describe, expect, it, vi } from 'vitest';
import { SKIP_WAITING } from '#lib/pwa/protocol.ts';
import { UpdatePrompt, whenIdle, type UpdateDeps } from '#lib/pwa/updatePrompt.ts';

function setup(overrides: Partial<UpdateDeps> = {}) {
  const worker = { postMessage: vi.fn() };
  let accept: (() => void) | null = null;
  let dismissed: (() => void) | null = null;
  const deps: UpdateDeps = {
    announce: vi.fn((run: () => void, dismiss: () => void) => {
      accept = run;
      dismissed = dismiss;
    }),
    waiting: () => worker,
    busy: () => false,
    reload: vi.fn(),
    sleep: () => Promise.resolve(),
    ...overrides
  };
  return {
    prompt: new UpdatePrompt(deps),
    deps,
    worker,
    accept: () => accept?.(),
    dismiss: () => dismissed?.()
  };
}

describe('whenIdle', () => {
  it('polls until the editor stops saving', async () => {
    let checks = 0;
    const sleep = vi.fn(() => Promise.resolve());
    await whenIdle(() => ++checks < 3, sleep);
    expect(sleep).toHaveBeenCalledTimes(2);
  });
});

describe('UpdatePrompt', () => {
  it('announces a waiting worker once per page', () => {
    const { prompt, deps } = setup();
    prompt.available();
    prompt.available();
    expect(deps.announce).toHaveBeenCalledOnce();
  });

  it('announces again after the toast is dismissed', () => {
    const { prompt, deps, dismiss } = setup();
    prompt.available();
    dismiss();
    prompt.available();
    expect(deps.announce).toHaveBeenCalledTimes(2);
  });

  it('does not announce again once the update was accepted', async () => {
    const { prompt, deps, accept, dismiss } = setup();
    prompt.available();
    accept();
    dismiss();
    prompt.available();
    await vi.waitFor(() => expect(deps.announce).toHaveBeenCalledOnce());
  });

  it('asks the waiting worker to take over when accepted', async () => {
    const { prompt, worker, deps } = setup();
    await prompt.accept();
    expect(worker.postMessage).toHaveBeenCalledWith(SKIP_WAITING);
    expect(deps.reload).not.toHaveBeenCalled();
  });

  it('wires the announced action to accept', async () => {
    const { prompt, worker, accept } = setup();
    prompt.available();
    accept();
    await vi.waitFor(() => expect(worker.postMessage).toHaveBeenCalledOnce());
  });

  it('waits for a pending save before taking over', async () => {
    let saving = true;
    const { prompt, worker } = setup({
      busy: () => saving,
      sleep: vi.fn(() => {
        saving = false;
        return Promise.resolve();
      })
    });
    const accepted = prompt.accept();
    expect(worker.postMessage).not.toHaveBeenCalled();
    await accepted;
    expect(worker.postMessage).toHaveBeenCalledOnce();
  });

  it('reloads directly when another tab already activated the update', async () => {
    const { prompt, deps } = setup({ waiting: () => null });
    await prompt.accept();
    expect(deps.reload).toHaveBeenCalledOnce();
  });

  it('reloads once after the controller changes', async () => {
    const { prompt, deps } = setup();
    await prompt.accept();
    await prompt.controllerChanged();
    await prompt.controllerChanged();
    expect(deps.reload).toHaveBeenCalledOnce();
  });

  it('ignores a controller change it did not ask for', async () => {
    const { prompt, deps } = setup();
    await prompt.controllerChanged();
    expect(deps.reload).not.toHaveBeenCalled();
  });

  it('holds the reload until a pending save clears', async () => {
    let saving = false;
    const sleep = vi.fn(() => {
      saving = false;
      return Promise.resolve();
    });
    const { prompt, deps } = setup({ busy: () => saving, sleep });
    await prompt.accept();
    saving = true;
    await prompt.controllerChanged();
    expect(sleep).toHaveBeenCalledOnce();
    expect(deps.reload).toHaveBeenCalledOnce();
  });
});
