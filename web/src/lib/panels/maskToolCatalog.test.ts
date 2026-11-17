import { describe, expect, it } from 'vitest';
import { aiTools } from './maskToolCatalog';

describe('aiTools', () => {
  it('follows the picker order with Background and Box select beside their models', () => {
    const tools = aiTools([
      { kind: 'click', installed: true },
      { kind: 'semantic', installed: true },
      { kind: 'depth', installed: false },
      { kind: 'sky', installed: true },
      { kind: 'subject', installed: true }
    ]);
    expect(tools.map((tool) => tool.label)).toEqual([
      'Subject',
      'Background',
      'Sky',
      'Depth',
      'Scene',
      'Click to select',
      'Box select'
    ]);
    expect(tools.find((tool) => tool.id === 'depth')?.installed).toBe(false);
    expect(tools.find((tool) => tool.id === 'semantic')?.scene).toBe(true);
  });

  it('hides the helpers of missing models', () => {
    expect(aiTools([{ kind: 'subject', installed: false }]).map((tool) => tool.id)).toEqual([
      'subject'
    ]);
  });
});
