import { describe, expect, it } from 'vitest';

const components = import.meta.glob<string>('/src/**/*.svelte', {
  query: '?raw',
  import: 'default',
  eager: true
});

const SLIDER_TAG = /<(?:SliderRow|EditSlider|MaskedEditSlider|DualRangeSlider)\b[\s\S]*?\/>/g;
const BOUND_ATTR = /(?<![\w-])(?:min|max|step)=|\{(?:min|max|step)\}/;
const RANGE_ATTR = /(?<![\w-])range=\{\s*(\w+)\s*\}|\{range\}/g;
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

function hasInlineRange(source: string): boolean {
  const names = catalogNames(source);
  return (source.match(SLIDER_TAG) ?? []).some(
    (tag) =>
      BOUND_ATTR.test(tag) ||
      [...tag.matchAll(RANGE_ATTR)].some(([, owner]) => !owner || !names.has(owner))
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
    ['a literal bound', '<SliderRow min={-100} max={100} />'],
    ['a catalog bound', "import { X } from '#lib/editor/params/x.ts';\n<EditSlider min={X.min} />"],
    ['a shorthand bound', '<SliderRow {min} />'],
    ['a string step', '<SliderRow step="0.5" />'],
    ['a range from elsewhere', "import { X } from '#lib/types/x.ts';\n<EditSlider range={X} />"],
    ['a local range', '<SliderRow range={local} />'],
    ['a shorthand range', '<SliderRow {range} />']
  ])('reject %s', (_, source) => {
    expect(hasInlineRange(source)).toBe(true);
  });

  it('accept ranges imported from the catalog', () => {
    const source =
      "import { KELVIN } from '#lib/editor/params/whiteBalance.ts';\n" +
      '<EditSlider range={KELVIN} defaultValue={asShot} />\n' +
      '<SliderRow range={param} />';
    expect(hasInlineRange(source)).toBe(false);
  });
});
