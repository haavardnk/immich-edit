import { logout, type AuthKind } from '#lib/api/auth.ts';
import { appearance } from '#lib/stores/appearance.svelte.ts';
import { recentSearches } from '#lib/stores/recentSearches.svelte.ts';
import { recentTags } from '#lib/stores/recentTags.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';

const AUTH_KIND_LABELS: Readonly<Record<AuthKind, string>> = {
  password: 'password',
  oauth: 'single sign-on',
  apikey: 'API key'
};

export function authKindLabel(kind: AuthKind): string {
  return AUTH_KIND_LABELS[kind];
}

export async function signOut(): Promise<void> {
  await logout().catch(() => undefined);
  session.clear();
  appearance.reset();
  recentSearches.clear();
  recentTags.clear();
  window.location.replace('/login');
}
