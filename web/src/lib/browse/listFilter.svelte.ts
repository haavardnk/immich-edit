import {
  filterByName,
  listFilterShown,
  listFilterStatus,
  type ListNoun
} from '#lib/browse/listFilter.ts';
import { firstFocusable } from '#lib/utils/focusable.ts';

export class ListFilterState<T> {
  #items: () => readonly T[];
  #nameOf: (item: T) => string;
  #noun: () => ListNoun;

  query = $state('');
  rows = $state<HTMLElement | null>(null);

  readonly available = $derived.by(() => listFilterShown(this.#items().length));
  readonly needle = $derived(this.available ? this.query.trim() : '');
  readonly visible = $derived.by(() =>
    this.needle ? filterByName(this.#items(), this.needle, this.#nameOf) : this.#items()
  );
  readonly status = $derived.by(() =>
    this.needle ? listFilterStatus(this.visible.length, this.#items().length, this.#noun()) : ''
  );

  constructor(items: () => readonly T[], nameOf: (item: T) => string, noun: () => ListNoun) {
    this.#items = items;
    this.#nameOf = nameOf;
    this.#noun = noun;
  }

  get label(): string {
    return `Filter ${this.#noun().many}`;
  }

  focusFirstRow(event: KeyboardEvent): void {
    if (event.key !== 'ArrowDown') return;
    const first = this.rows ? firstFocusable(this.rows) : null;
    if (!first) return;
    event.preventDefault();
    first.focus();
  }
}
