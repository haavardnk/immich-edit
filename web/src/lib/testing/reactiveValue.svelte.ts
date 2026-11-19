export interface Reactive<T> {
  value: T;
}

export function reactiveValue<T>(initial: T): Reactive<T> {
  let value = $state.raw(initial);
  return {
    get value(): T {
      return value;
    },
    set value(next: T) {
      value = next;
    }
  };
}
