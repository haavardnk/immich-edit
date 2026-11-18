export interface Flag {
  value: boolean;
}

export function reactiveFlag(initial: boolean): Flag {
  let value = $state(initial);
  return {
    get value(): boolean {
      return value;
    },
    set value(next: boolean) {
      value = next;
    }
  };
}
