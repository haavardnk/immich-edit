import { describe, expect, it } from 'vitest';

const components = import.meta.glob<string>('/src/**/*.svelte', {
  query: '?raw',
  import: 'default',
  eager: true
});

const METADATA_API = /\b(updateAsset|addTagToAsset|removeTagFromAsset|upsertTags)\b/;

describe('components', () => {
  it('never write photo metadata to Immich themselves', () => {
    const offenders = Object.entries(components)
      .filter(([, source]) => METADATA_API.test(source))
      .map(([path]) => path);
    expect(offenders).toEqual([]);
  });
});
