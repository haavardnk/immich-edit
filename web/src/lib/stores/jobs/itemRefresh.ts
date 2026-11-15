export class ItemRefresh {
  #load: (id: string) => Promise<void>;
  #pending = new Map<string, boolean>();

  constructor(load: (id: string) => Promise<void>) {
    this.#load = load;
  }

  request(id: string): void {
    if (this.#pending.has(id)) {
      this.#pending.set(id, true);
      return;
    }
    this.#pending.set(id, false);
    void this.#load(id)
      .catch(() => undefined)
      .finally(() => {
        const again = this.#pending.get(id) === true;
        this.#pending.delete(id);
        if (again) this.request(id);
      });
  }
}
