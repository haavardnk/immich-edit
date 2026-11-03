import type { KeybindContext, KeybindId } from '#lib/shortcuts/table.ts';

export type KeyGate = 'always' | 'outsideFields' | 'default';

export interface CommandTrigger {
  key: string;
}

export interface Command {
  id: string;
  contexts: readonly KeybindContext[];
  label?: string;
  icon?: string;
  bind?: KeybindId;
  gate?: KeyGate;
  active?: () => boolean;
  enabled?: () => boolean;
  unavailable?: () => string | null;
  yields?: () => boolean;
  run: (trigger?: CommandTrigger) => void;
  release?: () => void;
}
