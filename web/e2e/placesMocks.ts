import type { Request, Route } from '@playwright/test';

export interface MockSpot {
  lat: number;
  lon: number;
  city: string | null;
  state: string | null;
  country: string | null;
}

export interface MockSavedPlace extends MockSpot {
  id: string;
  name: string;
  created_at: string;
  used_at: string;
}

export interface MockRecentPlace extends MockSpot {
  id: string;
  used_at: string;
}

export interface MockPlaces {
  saved?: MockSavedPlace[];
  recent?: MockRecentPlace[];
}

export function savedPlace(id: string, name: string, spot: MockSpot, minute = 0): MockSavedPlace {
  const at = new Date(Date.UTC(2026, 10, 1, 12, minute)).toISOString();
  return { id, name, ...spot, created_at: at, used_at: at };
}

type Fulfil = Parameters<Route['fulfill']>[0];

function body(data: unknown, status = 200): Fulfil {
  return { status, contentType: 'application/json', body: JSON.stringify(data) };
}

function spotOf(input: Record<string, unknown>): MockSpot {
  const text = (value: unknown): string | null => (typeof value === 'string' ? value : null);
  return {
    lat: Number(input.lat),
    lon: Number(input.lon),
    city: text(input.city),
    state: text(input.state),
    country: text(input.country)
  };
}

export class PlacesMock {
  saved: MockSavedPlace[];
  recent: MockRecentPlace[];
  #seq = 0;

  constructor(seed: MockPlaces = {}) {
    this.saved = [...(seed.saved ?? [])];
    this.recent = [...(seed.recent ?? [])];
  }

  #stamp(): string {
    this.#seq += 1;
    return new Date(Date.UTC(2026, 10, 2, 0, 0, this.#seq)).toISOString();
  }

  #list(): MockSavedPlace[] {
    return [...this.saved].sort((a, b) => b.used_at.localeCompare(a.used_at));
  }

  async handle(route: Route, req: Request, path: string, method: string): Promise<boolean> {
    if (!path.startsWith('/api/places')) return false;
    if (path === '/api/places' && method === 'GET') {
      await route.fulfill(body({ saved: this.#list(), recent: this.recent }));
      return true;
    }
    if (path === '/api/places' && method === 'POST') {
      const input = req.postDataJSON() as Record<string, unknown>;
      const name = String(input.name).trim();
      if (this.saved.some((p) => p.name.toLowerCase() === name.toLowerCase())) {
        await route.fulfill(
          body({ code: 'conflict', message: `You already have a place called ${name}` }, 409)
        );
        return true;
      }
      const at = this.#stamp();
      const created = {
        id: `place-${this.#seq}`,
        name,
        ...spotOf(input),
        created_at: at,
        used_at: at
      };
      this.saved.push(created);
      await route.fulfill(body(created));
      return true;
    }
    if (path === '/api/places/recent') {
      if (method === 'DELETE') this.recent = [];
      else {
        const spot = spotOf(req.postDataJSON() as Record<string, unknown>);
        this.recent = [
          { id: `recent-${this.#seq + 1}`, ...spot, used_at: this.#stamp() },
          ...this.recent
        ].slice(0, 8);
      }
      await route.fulfill(method === 'DELETE' ? { status: 204, body: '' } : body(this.recent));
      return true;
    }
    const match = /^\/api\/places\/([^/]+)(\/used)?$/.exec(path);
    const idx = match ? this.saved.findIndex((p) => p.id === match[1]) : -1;
    const found = this.saved[idx];
    if (!match || !found) {
      await route.fulfill(body({ code: 'not_found', message: 'Not found' }, 404));
      return true;
    }
    if (match[2]) {
      this.saved[idx] = { ...found, used_at: this.#stamp() };
      await route.fulfill({ status: 204, body: '' });
      return true;
    }
    if (method === 'DELETE') {
      this.saved.splice(idx, 1);
      await route.fulfill({ status: 204, body: '' });
      return true;
    }
    const input = req.postDataJSON() as Record<string, unknown>;
    const updated = { ...found, name: String(input.name).trim(), ...spotOf(input) };
    this.saved[idx] = updated;
    await route.fulfill(body(updated));
    return true;
  }
}
