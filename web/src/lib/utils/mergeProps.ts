type Props = Record<string, unknown>;

type Handler = (...args: unknown[]) => unknown;

export function mergeProps(...sources: Props[]): Props {
  const merged: Props = {};
  for (const source of sources) {
    for (const key of Reflect.ownKeys(source)) {
      const value = (source as Record<string | symbol, unknown>)[key];
      const existing = (merged as Record<string | symbol, unknown>)[key];
      if (typeof existing === 'function' && typeof value === 'function') {
        const first = existing as Handler;
        const second = value as Handler;
        (merged as Record<string | symbol, unknown>)[key] = (...args: unknown[]) => {
          first(...args);
          second(...args);
        };
      } else if (key === 'class' && typeof existing === 'string' && typeof value === 'string') {
        merged[key] = `${existing} ${value}`;
      } else if (value !== undefined) {
        (merged as Record<string | symbol, unknown>)[key] = value;
      }
    }
  }
  return merged;
}
