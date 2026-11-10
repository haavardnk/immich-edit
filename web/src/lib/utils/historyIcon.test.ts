import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import type { HistoryDetailGroup } from '#lib/utils/history.ts';
import {
  mdiCropRotate,
  mdiImageOutline,
  mdiPaletteOutline,
  mdiTuneVariant,
  mdiViewGridOutline,
  mdiWhiteBalanceSunny
} from '@mdi/js';
import { describe, expect, it } from 'vitest';
import { historyIcon } from './historyIcon';

const ENTRY: EditHistoryEntry = {
  id: 1,
  manifest_hash: 'h1',
  deleted: false,
  edits: neutralEdits(),
  created_at: '2026-01-01T10:00:00Z',
  action: null,
  undone: false
};

function groups(...keys: string[]): HistoryDetailGroup[] {
  return keys.map((key) => ({ key, label: key, items: [] }));
}

describe('historyIcon', () => {
  it.each([
    [groups('tone'), mdiWhiteBalanceSunny],
    [groups('tone', 'curves'), mdiWhiteBalanceSunny],
    [groups('white_balance'), mdiPaletteOutline],
    [groups('geometry'), mdiCropRotate],
    [groups('tone', 'hsl'), mdiTuneVariant],
    [groups(), mdiTuneVariant]
  ])('maps %j to its section icon', (changed, icon) => {
    expect(historyIcon(ENTRY, changed)).toBe(icon);
  });

  it('uses the preset and original icons for those steps', () => {
    expect(historyIcon({ ...ENTRY, action: 'Preset: Portra' }, groups('tone'))).toBe(
      mdiViewGridOutline
    );
    expect(historyIcon({ ...ENTRY, deleted: true, edits: null }, [])).toBe(mdiImageOutline);
  });
});
