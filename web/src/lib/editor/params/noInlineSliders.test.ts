import { describe, expect, it } from 'vitest';

const components = import.meta.glob<string>('/src/**/*.svelte', {
  query: '?raw',
  import: 'default',
  eager: true
});

const SLIDER_TAG = /<(?:SliderRow|EditSlider|MaskedEditSlider|DualRangeSlider)\b[\s\S]*?\/>/g;
const RANGE_ATTR = /(?<![\w-])(?:min|max|step)=("[^"]*"|\{[^}]*\})|\{(?:min|max|step)\}/g;
const CATALOG_IMPORT = /import\s*\{([^}]*)\}\s*from\s*'#lib\/editor\/params\/[^']+'/g;

const FORWARDING_WRAPPERS = ['/src/lib/components/editor/controls/EditSlider.svelte'];

function catalogNames(source: string): Set<string> {
  const names = [...source.matchAll(CATALOG_IMPORT)].flatMap((match) =>
    (match[1] ?? '').split(',').map((name) =>
      name
        .replace(/^\s*type\s+/, '')
        .split(/\s+as\s+/)
        .pop()
        ?.trim()
    )
  );
  return new Set(['param', ...names.filter((name): name is string => !!name)]);
}

function fromCatalog(attribute: string, names: Set<string>): boolean {
  const owner = /^=\{\s*(\w+)\.(?:min|max|step)\s*\}$/.exec(attribute.replace(/^\w+/, ''))?.[1];
  return !!owner && names.has(owner);
}

function hasInlineRange(source: string): boolean {
  const names = catalogNames(source);
  return (source.match(SLIDER_TAG) ?? []).some((tag) =>
    (tag.match(RANGE_ATTR) ?? []).some((attribute) => !fromCatalog(attribute, names))
  );
}

describe('slider rows', () => {
  it('take their range from the parameter catalog', () => {
    const offenders = Object.entries(components)
      .filter(([path, source]) => !FORWARDING_WRAPPERS.includes(path) && hasInlineRange(source))
      .map(([path]) => path)
      .sort();
    expect(offenders).toEqual([]);
  });

  it.each([
    ['a literal', '<SliderRow min={-100} max={100} />'],
    ['a named constant', "import { X } from '#lib/types/x.ts';\n<EditSlider min={-LIMIT} />"],
    ['a local range', '<EditSlider min={local.min} />'],
    ['a shorthand prop', '<SliderRow {min} />'],
    ['a string', '<SliderRow step="0.5" />']
  ])('reject %s', (_, source) => {
    expect(hasInlineRange(source)).toBe(true);
  });

  it('accept ranges imported from the catalog', () => {
    const source =
      "import { KELVIN } from '#lib/editor/params/whiteBalance.ts';\n" +
      '<EditSlider min={KELVIN.min} max={KELVIN.max} step={KELVIN.step} />\n' +
      '<SliderRow min={param.min} max={param.max} />';
    expect(hasInlineRange(source)).toBe(false);
  });
});
