export interface Host<T> {
  attach: (impl: T) => () => void;
  get: () => T | null;
}

export function createHost<T>(): Host<T> {
  let current: T | null = null;
  return {
    attach: (impl) => {
      current = impl;
      return () => {
        if (current === impl) current = null;
      };
    },
    get: () => current
  };
}
