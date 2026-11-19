import type { Component } from 'svelte';
import type { SettingsPageId } from '#lib/settings/pages.ts';
import AccountPage from './AccountPage.svelte';
import AdministrationPage from './AdministrationPage.svelte';
import DiagnosticsPage from './DiagnosticsPage.svelte';
import PreferencesPage from './PreferencesPage.svelte';

export const PAGE_BODIES: Readonly<Record<SettingsPageId, Component>> = {
  preferences: PreferencesPage,
  account: AccountPage,
  administration: AdministrationPage,
  diagnostics: DiagnosticsPage
};
