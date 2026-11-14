import { layout } from '#lib/device/layout.svelte.ts';
import type { PillText } from '#lib/editor/pill.ts';
import { RETOUCH_LIMIT_TEXT } from '#lib/editor/retouch/text.ts';
import { keysFor } from '#lib/shortcuts/labels.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { mdiAlertOutline } from '@mdi/js';

const FLASH_MS = 1800;

class RetouchView {
  #visualize = $state(false);
  #limit = $state(false);
  #timer: ReturnType<typeof setTimeout> | null = null;

  get visualize(): boolean {
    return this.#visualize && ui.editorTab === 'retouch';
  }

  get pill(): PillText | null {
    if (ui.editorTab !== 'retouch') return null;
    if (this.#limit) {
      return { lead: RETOUCH_LIMIT_TEXT, label: '', tail: '', mono: false, icon: mdiAlertOutline };
    }
    if (!this.#visualize) return null;
    const keys = layout.mode === 'desktop' || layout.hasKeyboard;
    return {
      lead: '',
      label: 'Visualize spots',
      tail: '',
      mono: false,
      ...(keys ? { hint: `${keysFor('retouchVisualize')} turns it off` } : {})
    };
  }

  toggleVisualize = (): void => {
    this.#visualize = !this.#visualize;
  };

  flashLimit = (): void => {
    this.#limit = true;
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = setTimeout(() => {
      this.#limit = false;
      this.#timer = null;
    }, FLASH_MS);
  };
}

export const retouchView = new RetouchView();
