export const MEMBERSHIP_CAP = 100;

export function heldCounts(
  sources: readonly string[],
  heldBy: (id: string) => readonly { id: string }[] | null | undefined
): Map<string, number> | null {
  if (sources.length > MEMBERSHIP_CAP) return null;
  const counts = new Map<string, number>();
  for (const source of sources) {
    const held = heldBy(source);
    if (!held) return null;
    for (const { id } of held) counts.set(id, (counts.get(id) ?? 0) + 1);
  }
  return counts;
}
