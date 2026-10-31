import type { Model, ModelsResponse } from '#lib/api/models.ts';
import { describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({ list: vi.fn<() => Promise<ModelsResponse>>() }));

vi.mock('#lib/api/models.ts', () => ({ listModels: api.list }));

const { maskModels } = await import('./maskModels.svelte');

function model(id: string, kind: Model['kind'], installed: boolean): Model {
  return { id, kind, installed } as Model;
}

describe('mask model kinds', () => {
  it.each([
    {
      name: 'offers mask kinds only, leaving denoise to Detail',
      models: [
        model('birefnet', 'subject', true),
        model('nind', 'denoise', true),
        model('sam', 'click', false)
      ],
      kinds: [
        { kind: 'subject', installed: true },
        { kind: 'click', installed: false }
      ],
      unavailable: null
    },
    {
      name: 'reports no mask models when the catalog holds only denoise',
      models: [model('nind', 'denoise', true)],
      kinds: [],
      unavailable: 'No AI mask models are available on this server.'
    }
  ])('$name', async ({ models, kinds, unavailable }) => {
    api.list.mockResolvedValue({
      runtime: 'cpu',
      enabled: true,
      models,
      active: {},
      semantic_classes: []
    });

    await maskModels.load();

    expect(maskModels.kinds).toEqual(kinds);
    expect(maskModels.unavailable).toBe(unavailable);
  });
});
