import { KEYBINDS, type Keybind, type KeybindContext } from '#lib/shortcuts/table.ts';

export interface HelpGroup {
  title: string;
  binds: Keybind[];
}

export interface HelpSections {
  here: HelpGroup[];
  elsewhere: HelpGroup[];
}

export function helpSections(
  contexts: readonly KeybindContext[],
  query: string,
  keysOf: (bind: Keybind) => string
): HelpSections {
  const needle = query.trim().toLowerCase();
  const here = new Map<string, HelpGroup>();
  const elsewhere = new Map<string, HelpGroup>();
  for (const bind of KEYBINDS as readonly Keybind[]) {
    if (needle && !`${bind.label} ${keysOf(bind)}`.toLowerCase().includes(needle)) continue;
    const active = bind.contexts.some((context) => contexts.includes(context));
    const target = active ? here : elsewhere;
    const group = target.get(bind.group) ?? { title: bind.group, binds: [] };
    group.binds.push(bind);
    target.set(bind.group, group);
  }
  return { here: [...here.values()], elsewhere: [...elsewhere.values()] };
}
