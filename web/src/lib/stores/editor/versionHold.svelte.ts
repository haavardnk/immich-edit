import type { Version } from '#lib/editor/versions.ts';
import type { PillText } from '#lib/editor/pill.ts';

export interface VersionHoldCtx {
  current(): Version | null;
  url(id: string): string;
}

interface Held {
  version: Version;
  url: string;
}

export class VersionHold {
  private held = $state.raw<Held | null>(null);
  private ctx: VersionHoldCtx;

  constructor(ctx: VersionHoldCtx) {
    this.ctx = ctx;
  }

  get url(): string | null {
    return this.held?.url ?? null;
  }

  get heldId(): string | null {
    return this.held?.version.id ?? null;
  }

  get pill(): PillText | null {
    const held = this.held;
    const current = this.ctx.current();
    if (!held || !current) return null;
    return {
      lead: 'Holding',
      label: held.version.name,
      tail: `· release to return to ${current.name}`,
      mono: true
    };
  }

  canHold(id: string): boolean {
    const current = this.ctx.current();
    return current !== null && current.id !== id;
  }

  start(version: Version): boolean {
    if (!this.canHold(version.id)) return false;
    this.held = { version, url: this.ctx.url(version.id) };
    return true;
  }

  end(id?: string): void {
    if (!this.held || (id !== undefined && this.held.version.id !== id)) return;
    this.held = null;
  }
}
