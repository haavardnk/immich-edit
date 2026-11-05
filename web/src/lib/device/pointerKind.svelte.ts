export type PointerKind = 'mouse' | 'touch';

export function kindOf(pointerType: string): PointerKind {
  return pointerType === 'mouse' ? 'mouse' : 'touch';
}

class PointerKindStore {
  #kind = $state<PointerKind | null>(null);

  get kind(): PointerKind | null {
    return this.#kind;
  }

  get touch(): boolean {
    return this.#kind === 'touch';
  }

  note = (event: PointerEvent): void => {
    const kind = kindOf(event.pointerType);
    if (kind !== this.#kind) this.#kind = kind;
  };
}

export const pointerKind = new PointerKindStore();

const OPTIONS = { capture: true, passive: true } as const;

export function installPointerKind(): () => void {
  window.addEventListener('pointerdown', pointerKind.note, OPTIONS);
  window.addEventListener('pointermove', pointerKind.note, OPTIONS);
  return () => {
    window.removeEventListener('pointerdown', pointerKind.note, OPTIONS);
    window.removeEventListener('pointermove', pointerKind.note, OPTIONS);
  };
}
