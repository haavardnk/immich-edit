import type { CullState } from '#lib/browse/cullState.ts';
import { LABEL_NAMES } from '#lib/stores/labels.ts';

class Peek {
  id = $state<string | null>(null);
  #hovered: string | null = null;

  get hovered(): string | null {
    return this.#hovered;
  }

  hover(id: string): void {
    this.#hovered = id;
  }

  leave(id: string): void {
    if (this.#hovered === id) this.#hovered = null;
  }

  show(id: string): void {
    this.id = id;
  }

  hide(): void {
    this.id = null;
  }
}

export const peek = new Peek();

export function peekMarks(state: CullState, edited: boolean): string[] {
  return [
    state.isFavorite ? 'Favorite' : null,
    state.rejected ? 'Rejected' : null,
    state.label ? `${LABEL_NAMES[state.label]} label` : null,
    edited ? 'Edited' : null
  ].filter((mark): mark is string => mark !== null);
}
