import { layout } from '#lib/device/layout.svelte.ts';
import type { PillText } from '#lib/editor/pill.ts';
import { keysFor } from '#lib/shortcuts/labels.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { preferences } from '#lib/stores/preferences.svelte.ts';
import { DEFAULT_MASK_OVERLAY_COLOR } from '#lib/theme/maskOverlay.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

export type MaskOverlayMode = 'color' | 'colorOnBw' | 'whiteOnBlack';

export const MASK_OVERLAY_MODES: readonly { id: MaskOverlayMode; label: string }[] = [
  { id: 'color', label: 'Color' },
  { id: 'colorOnBw', label: 'Color on black and white' },
  { id: 'whiteOnBlack', label: 'White on black' }
];

const STORAGE_KEY = 'immich-edit:maskOverlay';
const FLASH_MS = 1500;

function storedMode(): MaskOverlayMode {
  const stored = readStored<{ mode: MaskOverlayMode }>(STORAGE_KEY)?.mode;
  return MASK_OVERLAY_MODES.find((entry) => entry.id === stored)?.id ?? 'color';
}

class MaskOverlayStore {
  mode = $state<MaskOverlayMode>(storedMode());
  #flashing = $state(false);
  #timer: ReturnType<typeof setTimeout> | null = null;

  get color(): string {
    return preferences.maskOverlayColor ?? DEFAULT_MASK_OVERLAY_COLOR;
  }

  get customColor(): string | null {
    return preferences.maskOverlayColor;
  }

  get label(): string {
    return MASK_OVERLAY_MODES.find((entry) => entry.id === this.mode)?.label ?? '';
  }

  get pill(): PillText | null {
    if (!this.#flashing) return null;
    const keys = layout.mode === 'desktop' || layout.hasKeyboard;
    return {
      lead: '',
      label: this.label,
      tail: '',
      mono: false,
      ...(keys ? { hint: `${keysFor('maskOverlayMode')} cycles overlay` } : {})
    };
  }

  setMode = (mode: MaskOverlayMode): void => {
    this.mode = mode;
    if (!editor.maskOverlayVisible) editor.toggleMaskOverlay();
    writeStored(STORAGE_KEY, { mode });
  };

  cycle = (): void => {
    const at = MASK_OVERLAY_MODES.findIndex((entry) => entry.id === this.mode);
    const next = MASK_OVERLAY_MODES[(at + 1) % MASK_OVERLAY_MODES.length]?.id ?? 'color';
    this.setMode(next);
    this.#flash();
  };

  setColor = (hex: string): Promise<void> =>
    preferences.setMaskOverlayColor(hex === DEFAULT_MASK_OVERLAY_COLOR ? null : hex);

  #flash(): void {
    this.#flashing = true;
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = setTimeout(() => {
      this.#flashing = false;
      this.#timer = null;
    }, FLASH_MS);
  }
}

export const maskOverlay = new MaskOverlayStore();
