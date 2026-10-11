import { whiteBalanceIsAsShot, type Edits, type WhiteBalance } from '$lib/types/edits';

const WHITE_BALANCE_OP = 'white_balance';

function num(value: unknown): number {
  return typeof value === 'number' ? value : 0;
}

export function encodeWhiteBalance(edits: Edits, ops: Record<string, unknown>): void {
  const wb = edits.basic.white_balance;
  if (whiteBalanceIsAsShot(wb)) return;
  ops[WHITE_BALANCE_OP] = { ...wb };
}

function parseWhiteBalance(raw: Record<string, unknown>): WhiteBalance | null {
  if (raw.mode === 'relative')
    return { mode: 'relative', temp: num(raw.temp), tint: num(raw.tint) };
  if (raw.mode === 'absolute' && typeof raw.kelvin === 'number') {
    return { mode: 'absolute', kelvin: raw.kelvin, tint: num(raw.tint) };
  }
  return null;
}

export function decodeWhiteBalance(
  ops: Record<string, unknown>,
  edits: Edits,
  schemaVersion: number
): void {
  const raw = ops[WHITE_BALANCE_OP];
  if (!raw || typeof raw !== 'object') return;
  const record = raw as Record<string, unknown>;
  if (schemaVersion < 5) {
    edits.basic.white_balance = {
      mode: 'relative',
      temp: num(record.temp),
      tint: 0 - num(record.tint)
    };
    return;
  }
  edits.basic.white_balance = parseWhiteBalance(record) ?? edits.basic.white_balance;
}

export function negateLegacyMaskTints(edits: Edits): void {
  for (const layer of edits.masks) {
    if (layer.edits.wb_tint !== undefined) layer.edits.wb_tint = 0 - layer.edits.wb_tint;
  }
}
