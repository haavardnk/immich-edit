import { untrack } from 'svelte';

export type Presence = {
  readonly mounted: boolean;
  exited: () => void;
};

export function presence(open: () => boolean): Presence {
  let shown = $state(untrack(open));

  $effect.pre(() => {
    if (open()) shown = true;
  });

  return {
    get mounted() {
      return open() || shown;
    },
    exited: () => {
      if (!open()) shown = false;
    }
  };
}
