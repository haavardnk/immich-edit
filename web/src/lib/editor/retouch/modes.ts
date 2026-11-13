import type { RetouchMode } from '#lib/types/edits.ts';
import { mdiBandage, mdiStamper } from '@mdi/js';

export const RETOUCH_MODES: readonly RetouchMode[] = ['heal', 'clone'];

export const RETOUCH_MODE_LABEL: Record<RetouchMode, string> = { heal: 'Heal', clone: 'Clone' };

export const RETOUCH_MODE_ICON: Record<RetouchMode, string> = {
  heal: mdiBandage,
  clone: mdiStamper
};
