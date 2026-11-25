import type { Camera } from '#lib/geotag/cameras.ts';

export class CameraClocks {
  #corrections = $state<Record<string, number>>({});
  #chosen = $state<string | null>(null);

  of(key: string): number {
    return this.#corrections[key] ?? 0;
  }

  set(key: string, ms: number): void {
    this.#corrections = { ...this.#corrections, [key]: ms };
  }

  choose(key: string): void {
    this.#chosen = key;
  }

  active(cameras: readonly Camera[]): string {
    const chosen = this.#chosen;
    if (chosen !== null && cameras.some((camera) => camera.key === chosen)) return chosen;
    return cameras[0]?.key ?? '';
  }

  reset(): void {
    this.#corrections = {};
    this.#chosen = null;
  }
}
