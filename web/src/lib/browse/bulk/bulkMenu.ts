import type { KeybindId } from '#lib/shortcuts/table.ts';

export interface BulkMenuRow {
  label: string;
  icon: string;
  bind: KeybindId | null;
  detail: string | null;
  disabled: boolean;
}

export interface BulkMenuAction extends BulkMenuRow {
  run: () => void;
}
