import {
  mdiAccountCircleOutline,
  mdiCogOutline,
  mdiInformationOutline,
  mdiTuneVariant
} from '@mdi/js';

export type SettingsPageId = 'preferences' | 'account' | 'administration' | 'diagnostics';

export interface SettingsPage {
  id: SettingsPageId;
  title: string;
  icon: string;
  adminOnly?: boolean;
}

export interface SettingsTarget {
  page: SettingsPageId;
  anchor?: string;
}

export const SETTINGS_PAGES: readonly SettingsPage[] = [
  { id: 'preferences', title: 'Preferences', icon: mdiTuneVariant },
  { id: 'account', title: 'Account', icon: mdiAccountCircleOutline },
  { id: 'administration', title: 'Administration', icon: mdiCogOutline, adminOnly: true },
  { id: 'diagnostics', title: 'Diagnostics', icon: mdiInformationOutline }
];

const LEGACY: Readonly<Record<string, SettingsTarget>> = {
  device: { page: 'preferences', anchor: 'device' },
  rendering: { page: 'preferences', anchor: 'renderer' },
  layout: { page: 'preferences', anchor: 'layout' },
  install: { page: 'preferences', anchor: 'install' },
  export: { page: 'preferences', anchor: 'save-automatically' },
  appearance: { page: 'preferences', anchor: 'appearance' },
  sessions: { page: 'account', anchor: 'sessions' },
  'export-presets': { page: 'account', anchor: 'export-presets' },
  users: { page: 'administration', anchor: 'users' },
  models: { page: 'administration', anchor: 'models' },
  defaults: { page: 'administration', anchor: 'lens-auto' },
  instance: { page: 'administration', anchor: 'instance' }
};

export function visiblePages(admin: boolean): SettingsPage[] {
  return SETTINGS_PAGES.filter((page) => admin || !page.adminOnly);
}

export function findPage(id: string | undefined): SettingsPage | null {
  return SETTINGS_PAGES.find((page) => page.id === id) ?? null;
}

export function settingsHref(page: SettingsPageId, anchor?: string): string {
  return anchor ? `/settings/${page}#${anchor}` : `/settings/${page}`;
}

export function legacyTarget(name: string): SettingsTarget | null {
  const page = findPage(name);
  return page ? { page: page.id } : (LEGACY[name] ?? null);
}
