import { beforeNavigate, goto } from '$app/navigation';
import { navigating, page } from '$app/state';
import { onMount, untrack } from 'svelte';
import { v4 as uuidv4 } from 'uuid';

const open = new Set<string>();

function staleLayer(): boolean {
  const current = page.state.backLayer;
  return current !== undefined && !open.has(current);
}

export function closeOnBack(close: () => void, stillOpen: () => boolean = () => false): void {
  const id = uuidv4();
  let entered = $state(false);
  let mounted = false;
  let leaving = false;

  function ours(): boolean {
    return page.state.backLayer === id;
  }

  function leave(): void {
    setTimeout(() => {
      if (navigating.type === null && ours()) history.back();
    });
  }

  function settle(): void {
    if (mounted) entered = ours();
    else leave();
  }

  function enter(): void {
    if (!mounted || leaving) return;
    const state = { ...page.state, backLayer: id };
    void goto(page.url.href, { shallow: true, replace: staleLayer(), state }).then(settle, settle);
  }

  function arrive(): void {
    leaving = false;
    entered = false;
    enter();
  }

  beforeNavigate((navigation) => {
    if (navigation.shallow) return;
    if (navigation.type === 'link' && !navigation.willUnload && navigation.to && ours()) {
      navigation.cancel();
      void goto(navigation.to.url, { replace: true });
      return;
    }
    leaving = true;
    navigation.complete.then(arrive, () => {
      leaving = false;
    });
  });

  $effect(() => {
    if (!entered || leaving || ours()) return;
    entered = false;
    untrack(close);
    if (untrack(stillOpen)) enter();
  });

  onMount(() => {
    mounted = true;
    open.add(id);
    void (navigating.complete ?? Promise.resolve()).then(enter, enter);
    return () => {
      mounted = false;
      open.delete(id);
      if (entered) leave();
    };
  });
}
