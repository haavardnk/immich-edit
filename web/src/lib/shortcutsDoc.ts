import { KEYBINDS, formatChord, type Keybind, type KeybindContext } from './keybinds';

const CONTEXT_LABELS: Record<KeybindContext, string> = {
  global: 'Anywhere',
  grid: 'Grid',
  loupe: 'Loupe',
  compare: 'Compare',
  survey: 'Survey',
  editor: 'Editor',
  masks: 'Masks',
  retouch: 'Retouch'
};

const PREAMBLE = `---
layout: default
title: Keyboard shortcuts
nav_order: 4
permalink: /shortcuts/
---

<!-- Generated from web/src/lib/keybinds.ts. Run \`npm run docs:shortcuts\` in web/ to update. -->

# Keyboard shortcuts

Press \`?\` anywhere in immich-edit to open the same list as a searchable dialog. On macOS, press
Command wherever a shortcut below says \`Ctrl\`.

Shortcuts do not run while focus is in a text field or another typing control.
`;

type Row = readonly [string, string, string];

const HEADERS: Row = ['Keys', 'Action', 'Available in'];

function keysCell(bind: Keybind): string {
  if (bind.keys.length === 0) {
    if (!bind.display) return '';
    return `\`${typeof bind.display === 'string' ? bind.display : bind.display(false)}\``;
  }
  return bind.keys.map((spec) => `\`${formatChord(spec, false)}\``).join(' / ');
}

function contextsCell(bind: Keybind): string {
  return bind.contexts.map((context) => CONTEXT_LABELS[context]).join(', ');
}

function table(binds: readonly Keybind[]): string {
  const rows: Row[] = binds.map((bind) => [keysCell(bind), bind.label, contextsCell(bind)]);
  const all = [HEADERS, ...rows];
  const widths: readonly [number, number, number] = [
    Math.max(...all.map((row) => row[0].length)),
    Math.max(...all.map((row) => row[1].length)),
    Math.max(...all.map((row) => row[2].length))
  ];
  const line = (row: Row): string =>
    `| ${row[0].padEnd(widths[0])} | ${row[1].padEnd(widths[1])} | ${row[2].padEnd(widths[2])} |`;
  const divider = `| ${'-'.repeat(widths[0])} | ${'-'.repeat(widths[1])} | ${'-'.repeat(widths[2])} |`;
  return [line(HEADERS), divider, ...rows.map(line)].join('\n');
}

export function renderShortcutsDoc(): string {
  const binds = KEYBINDS as readonly Keybind[];
  const groups = [...new Set(binds.map((bind) => bind.group))];
  const sections = groups.map(
    (group) => `## ${group}\n\n${table(binds.filter((bind) => bind.group === group))}`
  );
  return `${PREAMBLE}\n${sections.join('\n\n')}\n`;
}
