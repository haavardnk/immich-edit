import type { KeybindContext, KeybindId } from '#lib/shortcuts/table.ts';

export type KeyGate = 'always' | 'outsideFields' | 'outsideControls' | 'outsideWidgets' | 'default';

export interface CommandTrigger {
  key: string;
  code: string;
  shiftKey: boolean;
  repeat: boolean;
}

export interface Command {
  id: string;
  contexts: readonly KeybindContext[];
  label?: string | (() => string);
  short?: string;
  icon?: string;
  bind?: KeybindId;
  gate?: KeyGate;
  active?: (trigger: CommandTrigger) => boolean;
  enabled?: () => boolean;
  pressed?: () => boolean;
  unavailable?: () => string | null;
  yields?: (trigger: CommandTrigger) => boolean;
  run: (trigger?: CommandTrigger) => void;
  release?: () => void;
}

export type IconCommand = Command & { icon: string };
