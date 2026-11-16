export type SetupStep = 'server' | 'admin';

export type AdminCredentials = { email: string; password: string } | { api_key: string };

export const SETUP_STEPS: ReadonlyArray<{ id: SetupStep; label: string }> = [
  { id: 'server', label: 'Immich server' },
  { id: 'admin', label: 'Administrator' }
];
