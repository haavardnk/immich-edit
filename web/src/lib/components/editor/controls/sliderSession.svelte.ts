import { keyHeld, latched, type ModifierKeys } from '#lib/input/modifiers.svelte.ts';
import { editor } from '#lib/stores/editor.svelte.ts';

export type SliderSessionHost = {
  value: () => number;
  disabled: () => boolean;
  write: (value: number) => void;
  live: (value: number) => void;
  commit: () => void;
  previewStart?: () => void;
  previewEnd?: () => void;
  dragStart?: () => void;
  dragEnd?: () => void;
};

export class SliderSession {
  previewKey = $state(false);
  #host: SliderSessionHost;
  #start: number | null = null;
  #previewing = false;
  #listening: AbortController | null = null;

  constructor(host: SliderSessionHost) {
    this.#host = host;
  }

  get previewHeld(): boolean {
    return this.previewKey || latched('preview');
  }

  begin = (keys?: ModifierKeys): void => {
    if (this.#start !== null) return;
    this.#start = this.#host.value();
    this.previewKey = keys ? keyHeld('preview', keys) : false;
    editor.beginDrag();
    this.#host.dragStart?.();
    if (keys) this.#watchKeys();
    this.sync();
  };

  set = (next: number): void => {
    this.#host.write(next);
    if (this.#previewing) this.#host.previewStart?.();
    else this.#host.live(next);
  };

  end = (): void => {
    const start = this.#start;
    if (start === null) return;
    this.#start = null;
    this.#listening?.abort();
    this.#listening = null;
    editor.endDrag();
    this.#host.dragEnd?.();
    this.sync();
    if (this.#host.value() !== start) this.#host.commit();
  };

  sync = (): void => {
    const { previewStart, previewEnd } = this.#host;
    if (!previewStart || !previewEnd) return;
    const want = this.#start !== null && this.previewHeld && !this.#host.disabled();
    if (want === this.#previewing) return;
    this.#previewing = want;
    if (want) previewStart();
    else previewEnd();
  };

  #watchKeys(): void {
    this.#listening = new AbortController();
    const onKey = (e: KeyboardEvent): void => {
      this.previewKey = keyHeld('preview', e);
      this.sync();
    };
    const options = { signal: this.#listening.signal };
    window.addEventListener('keydown', onKey, options);
    window.addEventListener('keyup', onKey, options);
  }
}
