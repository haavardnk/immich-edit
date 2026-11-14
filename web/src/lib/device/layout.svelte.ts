import {
  LAYOUT_CHOICES,
  resolveLayout,
  type Layout,
  type LayoutChoice,
  type LayoutMode
} from '#lib/device/layoutRules.ts';
import { isTypingTarget } from '#lib/shortcuts/targets.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';
import { MediaQuery } from 'svelte/reactivity';
import { innerHeight, innerWidth } from 'svelte/reactivity/window';

const STORAGE_KEY = 'immich-edit:layout';
const FALLBACK_WIDTH = 1920;
const FALLBACK_HEIGHT = 1080;

type Persisted = { choice: LayoutChoice };

function storedChoice(): LayoutChoice {
  const choice = readStored<Persisted>(STORAGE_KEY)?.choice;
  return LAYOUT_CHOICES.find((option) => option === choice) ?? 'auto';
}

class LayoutStore {
  #choice = $state<LayoutChoice>(storedChoice());
  #hasKeyboard = $state(false);
  #coarse = new MediaQuery('pointer: coarse');
  #resolved: Layout = $derived(
    resolveLayout({
      width: innerWidth.current ?? FALLBACK_WIDTH,
      height: innerHeight.current ?? FALLBACK_HEIGHT,
      coarse: this.#coarse.current,
      choice: this.#choice
    })
  );

  get choice(): LayoutChoice {
    return this.#choice;
  }

  get mode(): LayoutMode {
    return this.#resolved.mode;
  }

  get wide(): boolean {
    return this.#resolved.wide;
  }

  get landscape(): boolean {
    return this.#resolved.landscape;
  }

  get compact(): boolean {
    return this.#resolved.compact;
  }

  get phone(): boolean {
    return this.#resolved.phone;
  }

  get hasKeyboard(): boolean {
    return this.#hasKeyboard;
  }

  setChoice(choice: LayoutChoice): void {
    if (this.#choice === choice) return;
    this.#choice = choice;
    writeStored(STORAGE_KEY, { choice } satisfies Persisted);
  }

  noteHardwareKey(): void {
    this.#hasKeyboard = true;
  }
}

export const layout = new LayoutStore();

export function installLayout(): () => void {
  const onKeydown = (e: KeyboardEvent): void => {
    if (isTypingTarget(e)) return;
    layout.noteHardwareKey();
    window.removeEventListener('keydown', onKeydown, { capture: true });
  };
  window.addEventListener('keydown', onKeydown, { capture: true });
  return () => window.removeEventListener('keydown', onKeydown, { capture: true });
}
