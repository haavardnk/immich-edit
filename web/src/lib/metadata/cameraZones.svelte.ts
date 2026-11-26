import type { ZoneChoice } from '#lib/metadata/captureTimes.ts';

export interface CameraZone {
  takenIn: string | null;
  clock: string | null;
}

const AS_SHOT: CameraZone = { takenIn: null, clock: null };

export class CameraZones {
  #zones = $state<Record<string, CameraZone>>({});

  constructor(readonly fallback: string) {}

  of(key: string): CameraZone {
    return this.#zones[key] ?? AS_SHOT;
  }

  choice(key: string): ZoneChoice {
    const { takenIn, clock } = this.of(key);
    return { takenIn, clock: takenIn === null ? null : clock, fallback: this.fallback };
  }

  common(keys: readonly string[]): CameraZone | null {
    const [first, ...rest] = keys.map((key) => this.of(key));
    if (!first) return AS_SHOT;
    const same = rest.every((zone) => zone.takenIn === first.takenIn && zone.clock === first.clock);
    return same ? first : null;
  }

  setTakenIn(keys: readonly string[], takenIn: string | null): void {
    this.#update(keys, (zone) => ({ takenIn, clock: takenIn === null ? null : zone.clock }));
  }

  setClock(keys: readonly string[], clock: string | null): void {
    this.#update(keys, (zone) => ({ ...zone, clock }));
  }

  reset(): void {
    this.#zones = {};
  }

  #update(keys: readonly string[], change: (zone: CameraZone) => CameraZone): void {
    const next = { ...this.#zones };
    for (const key of keys) next[key] = change(this.of(key));
    this.#zones = next;
  }
}
