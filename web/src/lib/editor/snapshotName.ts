const PATTERN = /^Snapshot (\d+)$/;

export function nextSnapshotName(names: Iterable<string>): string {
  const used = new Set<number>();
  for (const name of names) {
    const match = PATTERN.exec(name.trim());
    if (match?.[1]) used.add(Number(match[1]));
  }
  let n = 1;
  while (used.has(n)) n++;
  return `Snapshot ${n}`;
}
