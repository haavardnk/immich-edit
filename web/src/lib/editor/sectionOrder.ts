export interface SectionEntry {
  readonly id: string;
  readonly hidden: boolean;
}

export function orderSections(
  defaults: readonly string[],
  stored: readonly SectionEntry[] | null
): SectionEntry[] {
  const seen = new Set<string>();
  const entries: SectionEntry[] = [];
  for (const entry of stored ?? []) {
    if (!defaults.includes(entry.id) || seen.has(entry.id)) continue;
    seen.add(entry.id);
    entries.push({ id: entry.id, hidden: entry.hidden });
  }
  defaults.forEach((id, index) => {
    if (seen.has(id)) return;
    const before = defaults.slice(0, index).findLast((prior) => seen.has(prior));
    const at = before === undefined ? 0 : entries.findIndex((entry) => entry.id === before) + 1;
    entries.splice(at, 0, { id, hidden: false });
    seen.add(id);
  });
  return entries;
}

export function storedSections(
  entries: readonly SectionEntry[],
  defaults: readonly string[]
): SectionEntry[] | null {
  const isDefault = entries.every((entry, index) => !entry.hidden && entry.id === defaults[index]);
  return isDefault && entries.length === defaults.length
    ? null
    : entries.map((entry) => ({ ...entry }));
}

export function reorderSections(
  entries: readonly SectionEntry[],
  order: readonly string[]
): SectionEntry[] {
  const moved = order
    .map((id) => entries.find((entry) => entry.id === id))
    .filter((entry): entry is SectionEntry => entry !== undefined);
  let next = 0;
  return entries.map((entry) => {
    if (!order.includes(entry.id)) return entry;
    const replacement = moved[next] ?? entry;
    next += 1;
    return replacement;
  });
}

export function toggleSection(entries: readonly SectionEntry[], id: string): SectionEntry[] {
  return entries.map((entry) => (entry.id === id ? { id, hidden: !entry.hidden } : entry));
}

export function moveWithin(order: readonly string[], id: string, to: number): string[] {
  const next = order.filter((entry) => entry !== id);
  next.splice(Math.max(0, Math.min(to, next.length)), 0, id);
  return next;
}
