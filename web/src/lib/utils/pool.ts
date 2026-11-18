export async function runPool<T>(
  items: readonly T[],
  limit: number,
  run: (item: T) => Promise<void>
): Promise<void> {
  const queue = items.values();
  const worker = async (): Promise<void> => {
    for (let step = queue.next(); !step.done; step = queue.next()) await run(step.value);
  };
  await Promise.all(Array.from({ length: Math.max(1, Math.min(limit, items.length)) }, worker));
}

export async function countFailures<T>(
  items: readonly T[],
  limit: number,
  run: (item: T) => Promise<boolean>
): Promise<number> {
  let failed = 0;
  await runPool(items, limit, async (item) => {
    const ok = await run(item).catch(() => false);
    if (!ok) failed += 1;
  });
  return failed;
}
