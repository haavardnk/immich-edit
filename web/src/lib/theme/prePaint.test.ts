import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { CACHE_VERSION, THEME_CACHE_KEY } from './cache';
import { THEME_STYLE_ID } from './document';
import { THEME_REVISION_PLACEHOLDER } from './revision';

const appHtml = readFileSync(fileURLToPath(new URL('../../app.html', import.meta.url)), 'utf8');

describe('pre-paint script', () => {
  it('reads the same cache, version and style element as the runtime', () => {
    expect(appHtml).toContain(`localStorage.getItem('${THEME_CACHE_KEY}')`);
    expect(appHtml).toContain(`cache.version !== ${CACHE_VERSION}`);
    expect(appHtml).toContain(`data-theme-revision="${THEME_REVISION_PLACEHOLDER}"`);
    expect(appHtml).toContain("cache.revision !== root.getAttribute('data-theme-revision')");
    expect(appHtml).toContain(`style.id = '${THEME_STYLE_ID}'`);
    expect(appHtml).toContain(`localStorage.getItem('immich-ui-theme')`);
  });
});
