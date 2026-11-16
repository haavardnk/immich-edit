import { logout } from '#lib/api/auth.ts';
import { appearance } from '#lib/stores/appearance.svelte.ts';
import { preferences } from '#lib/stores/preferences.svelte.ts';
import { recentSearches } from '#lib/stores/recentSearches.svelte.ts';
import { recentTags } from '#lib/stores/recentTags.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';

export async function signOut(): Promise<void> {
  await logout().catch(() => undefined);
  session.clear();
  appearance.reset();
  preferences.reset();
  recentSearches.clear();
  recentTags.clear();
  window.location.replace('/login');
}
