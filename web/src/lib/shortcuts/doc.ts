import { formatChord } from './chord';
import { CONTEXT_NAMES } from './stacks';
import { KEYBINDS, type Keybind } from './table';

const PREAMBLE = `---
title: Keyboard shortcuts
description: Default keyboard shortcuts and how to customize them in immich-edit.
---

Press \`?\` anywhere to open this list as a searchable dialog. On macOS, press Command wherever a
shortcut says \`Ctrl\`.

Click any key in the dialog to rebind it; conflicting keys can be swapped. Ranges, arrow keys,
\`Esc\`, and pointer gestures stay fixed. Custom bindings save to your account.

\`Shift\` + letter opens a tool or panel, a plain letter acts on the photo, and \`Ctrl\` runs a
command. Symbol keys (\`[\`, \`{\`, \`\\\`) work on any keyboard layout. Shortcuts do not run while
typing in a text field.
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
  return bind.contexts.map((context) => CONTEXT_NAMES[context]).join(', ');
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
