export class CameraClocks {
  #corrections = $state<Record<string, number>>({});

  of(key: string): number {
    return this.#corrections[key] ?? 0;
  }

  set(key: string, ms: number): void {
    this.#corrections = { ...this.#corrections, [key]: ms };
  }

  reset(): void {
    this.#corrections = {};
  }
}
