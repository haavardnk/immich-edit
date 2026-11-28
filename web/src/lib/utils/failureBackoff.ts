export class FailureBackoff {
  readonly #ms: number;
  readonly #failedAt = new Map<string, number>();

  constructor(ms: number) {
    this.#ms = ms;
  }

  waiting(key: string): boolean {
    const at = this.#failedAt.get(key);
    if (at === undefined) return false;
    if (Date.now() - at < this.#ms) return true;
    this.#failedAt.delete(key);
    return false;
  }

  fail(keys: Iterable<string>): void {
    const now = Date.now();
    for (const [key, at] of this.#failedAt) if (now - at >= this.#ms) this.#failedAt.delete(key);
    for (const key of keys) this.#failedAt.set(key, now);
  }

  forgive(keys: Iterable<string>): void {
    for (const key of keys) this.#failedAt.delete(key);
  }

  clear(): void {
    this.#failedAt.clear();
  }
}
