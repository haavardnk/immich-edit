import { describe, it, expect } from 'vitest';
import type { AuthProviders } from '$lib/api/auth';
import { callbackError, isOAuthCallback, redirectUri, safeNext, shouldAutoLaunch } from './oauth';

function providers(overrides: Partial<AuthProviders> = {}): AuthProviders {
  return {
    oauth: true,
    password_login: true,
    auto_launch: true,
    button_text: 'Login with OAuth',
    degraded: false,
    ...overrides
  };
}

describe('isOAuthCallback', () => {
  it.each([
    ['https://edit.test/login?code=abc&state=xyz', true],
    ['https://edit.test/login?error=access_denied', true],
    ['https://edit.test/login?next=%2Falbums', false],
    ['https://edit.test/login', false]
  ])('%s -> %s', (url, expected) => {
    expect(isOAuthCallback(new URL(url))).toBe(expected);
  });
});

describe('callbackError', () => {
  it('joins the code and the description', () => {
    const url = new URL('https://edit.test/login?error=access_denied&error_description=Nope');
    expect(callbackError(url)).toBe('access_denied: Nope');
  });

  it('is null without an error', () => {
    expect(callbackError(new URL('https://edit.test/login?code=abc'))).toBeNull();
  });
});

describe('shouldAutoLaunch', () => {
  it.each([
    ['https://edit.test/login', providers(), true],
    ['https://edit.test/login', providers({ oauth: false }), false],
    ['https://edit.test/login', providers({ auto_launch: false }), false],
    ['https://edit.test/login?autoLaunch=0', providers(), false],
    ['https://edit.test/login?password=1', providers(), false],
    ['https://edit.test/login?code=abc', providers(), false],
    ['https://edit.test/login?error=access_denied', providers(), false]
  ])('%s -> %s', (url, config, expected) => {
    expect(shouldAutoLaunch(new URL(url), config)).toBe(expected);
  });
});

describe('redirectUri', () => {
  it('drops the query and the fragment', () => {
    const url = new URL('https://edit.test/login?next=%2Falbums#frag');
    expect(redirectUri(url)).toBe('https://edit.test/login');
  });
});

describe('safeNext', () => {
  it.each([
    ['/albums/42', '/albums/42'],
    ['/albums?sort=taken#top', '/albums?sort=taken#top'],
    ['//evil.test', '/'],
    ['/\\evil.test', '/'],
    ['/\t/evil.test', '/'],
    ['/\n/evil.test', '/'],
    ['/\r/evil.test', '/'],
    ['https://evil.test', '/'],
    ['javascript:alert(1)', '/'],
    [null, '/']
  ])('%s -> %s', (raw, expected) => {
    expect(safeNext(raw)).toBe(expected);
  });
});
