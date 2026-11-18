export type EffectLog<T> = { values: T[]; stop: () => void };

export function effectLog<T>(read: () => T): EffectLog<T> {
  const values: T[] = [];
  const stop = $effect.root(() => {
    $effect(() => {
      values.push(read());
    });
  });
  return { values, stop };
}
