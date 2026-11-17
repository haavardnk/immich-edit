import { describe, expect, it } from 'vitest';
import { aiTools, quickTools } from './maskToolCatalog';

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

describe('quickTools', () => {
  it('leads with installed AI tools and fills six slots with drawing tools', () => {
    const labels = (kinds: Parameters<typeof quickTools>[0]) =>
      quickTools(kinds).map((tool) => tool.label);
    expect(
      labels([
        { kind: 'sky', installed: true },
        { kind: 'subject', installed: true }
      ])
    ).toEqual(['Subject', 'Sky', 'Background', 'Brush', 'Linear', 'Radial']);
    expect(labels([{ kind: 'subject', installed: false }])).toEqual([
      'Brush',
      'Linear',
      'Radial',
      'Polygon',
      'Luminance',
      'Color'
    ]);
  });
});
