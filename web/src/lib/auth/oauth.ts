import type { AuthProviders } from '$lib/api/auth';

// Resolving against an unreachable origin catches escapes the browser would make itself.
const NEXT_BASE = 'https://next.invalid';

export function isOAuthCallback(url: URL): boolean {
  return url.searchParams.has('code') || url.searchParams.has('error');
}

export function callbackError(url: URL): string | null {
  const error = url.searchParams.get('error');
  if (!error) return null;
  const description = url.searchParams.get('error_description');
  return description ? `${error}: ${description}` : error;
}

export function shouldAutoLaunch(url: URL, providers: AuthProviders): boolean {
  if (!providers.oauth || !providers.auto_launch) return false;
  if (isOAuthCallback(url)) return false;
  if (url.searchParams.get('autoLaunch') === '0') return false;
  if (url.searchParams.get('password') === '1') return false;
  return true;
}

export function redirectUri(url: URL): string {
  return `${url.origin}${url.pathname}`;
}

export function safeNext(raw: string | null): string {
  if (!raw) return '/';
  let url: URL;
  try {
    url = new URL(raw, NEXT_BASE);
  } catch {
    return '/';
  }
  if (url.origin !== NEXT_BASE) return '/';
  return `${url.pathname}${url.search}${url.hash}`;
}
