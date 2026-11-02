import { FAMILIES } from '#lib/theme/registry.ts';
import { THEME_REVISION_PLACEHOLDER, themeRevision } from '#lib/theme/revision.ts';
import type { RequestEvent } from '@sveltejs/kit';
import type { ResolveOptions } from '@sveltejs/kit/hooks';
import { describe, expect, it } from 'vitest';
import { handle } from './hooks.server';

describe('server handle', () => {
  it('stamps the theme revision into the page shell', async () => {
    let transformed: string | undefined;
    const resolve = async (_event: RequestEvent, options?: ResolveOptions): Promise<Response> => {
      transformed = await options?.transformPageChunk?.({
        html: `<html data-theme-revision="${THEME_REVISION_PLACEHOLDER}">`,
        done: true
      });
      return new Response(transformed);
    };
    const event = {} as RequestEvent;

    await handle({ event, resolve });

    expect(transformed).toBe(`<html data-theme-revision="${themeRevision(FAMILIES)}">`);
  });
});
