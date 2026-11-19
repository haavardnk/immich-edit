import type { SettingsEntry } from '#lib/settings/settingsIndex.ts';
import { settingsHref, type SettingsPage } from '#lib/settings/pages.ts';
import { foldText } from '#lib/utils/textMatch.ts';

export interface SettingsHit {
  entry: SettingsEntry;
  page: SettingsPage;
  href: string;
}

export interface SettingsSearchScope {
  pages: readonly SettingsPage[];
  installable: boolean;
}

function haystack(entry: SettingsEntry, page: SettingsPage): string {
  return foldText([page.title, entry.title, entry.detail, ...(entry.keywords ?? [])].join(' '));
}

export function searchSettings(
  query: string,
  entries: readonly SettingsEntry[],
  scope: SettingsSearchScope
): SettingsHit[] {
  const words = foldText(query).split(/\s+/).filter(Boolean);
  if (words.length === 0) return [];
  return entries.flatMap((entry) => {
    const page = scope.pages.find((candidate) => candidate.id === entry.page);
    if (!page) return [];
    if (entry.installable && !scope.installable) return [];
    if (!words.every((word) => haystack(entry, page).includes(word))) return [];
    return [{ entry, page, href: settingsHref(page.id, entry.anchor) }];
  });
}
