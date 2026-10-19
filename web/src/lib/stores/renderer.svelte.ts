import { reportClientError } from '$lib/api/clientLog';
import { ClientRenderer } from '$lib/render/clientRenderer';
import type { GpuLimits, TickSample } from '$lib/render/protocol';
import { errorMessage } from '$lib/utils/errors';
import { readStored, writeStored } from '$lib/utils/storage';
import { blend, decide, msPerMegapixel, type Verdict } from './rendererGate';

export type RendererChoice = 'auto' | 'browser' | 'server';
export type RendererState = 'idle' | 'starting' | 'measuring' | 'browser' | 'server';

export interface RenderNeed {
  benchmarkEdge: number;
  maxEdge: number;
}

export type RendererBinding =
  | { kind: 'browser'; client: ClientRenderer }
  | { kind: 'server' }
  | { kind: 'wait'; ready: Promise<unknown> };

interface Stored {
  choice: RendererChoice;
  browserMs: number;
  serverMs: number;
}

const STORAGE_KEY = 'immich-edit:renderer';
const CHOICES: RendererChoice[] = ['auto', 'browser', 'server'];
const SERVER_CHOSEN = 'server selected';
const BROWSER_CHOSEN = 'browser selected';
const SERVER: RendererBinding = { kind: 'server' };

function storedMs(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) && value > 0 ? value : null;
}

class RendererStore {
  choice = $state<RendererChoice>('auto');
  state = $state<RendererState>('idle');
  reason = $state<string | null>(null);
  adapter = $state<string | null>(null);
  limits = $state<GpuLimits | null>(null);
  initMs = $state<number | null>(null);
  lastRenderMs = $state<number | null>(null);
  browserMs = $state<number | null>(null);
  serverMs = $state<number | null>(null);

  private client: ClientRenderer | null = null;
  private starting: Promise<ClientRenderer | null> | null = null;
  private failure: string | null = null;
  private edge = 0;

  constructor() {
    const stored = readStored<Stored>(STORAGE_KEY);
    const choice = stored?.choice;
    if (choice && CHOICES.includes(choice)) this.choice = choice;
    this.browserMs = storedMs(stored?.browserMs);
    this.serverMs = storedMs(stored?.serverMs);
    this.refresh();
  }

  get current(): ClientRenderer | null {
    return this.client;
  }

  get measuresServer(): boolean {
    return this.choice === 'auto' && !this.failure;
  }

  bind(need: RenderNeed): RendererBinding {
    this.edge = need.maxEdge;
    if (this.choice === 'server' || this.failure) {
      this.refresh();
      return SERVER;
    }
    if (!this.client) {
      const browser = this.wantsBrowser();
      const ready = this.launch(browser ? null : need.benchmarkEdge);
      this.refresh();
      return browser ? { kind: 'wait', ready } : SERVER;
    }
    this.refresh();
    return this.wantsBrowser() ? { kind: 'browser', client: this.client } : SERVER;
  }

  setChoice(choice: RendererChoice): void {
    if (this.choice === choice) return;
    this.choice = choice;
    this.failure = null;
    this.release();
    this.persist();
    this.refresh();
  }

  fail(err: unknown): void {
    reportClientError('render_worker', err);
    this.release();
    this.failure = `browser failed: ${errorMessage(err)}`;
    this.refresh();
  }

  recordRender(sample: TickSample): void {
    this.lastRenderMs = sample.ms;
    this.recordBrowser(sample);
  }

  recordServer(sample: TickSample): void {
    const ms = msPerMegapixel(sample);
    if (ms === null) return;
    this.serverMs = blend(this.serverMs, ms);
    this.persist();
    this.refresh();
  }

  private recordBrowser(sample: TickSample): void {
    const ms = msPerMegapixel(sample);
    if (ms === null) return;
    this.browserMs = blend(this.browserMs, ms);
    this.persist();
    this.refresh();
  }

  private verdict(): Verdict {
    return decide({
      browserMs: this.browserMs,
      serverMs: this.serverMs,
      limits: this.limits,
      edge: this.edge
    });
  }

  private wantsBrowser(): boolean {
    return this.choice === 'browser' || this.verdict().use === 'browser';
  }

  private launch(edge: number | null): Promise<ClientRenderer | null> {
    this.starting ??= this.boot(edge);
    return this.starting;
  }

  private async boot(edge: number | null): Promise<ClientRenderer | null> {
    try {
      const client = await ClientRenderer.start(edge);
      if (this.choice === 'server') {
        client.dispose();
        return null;
      }
      this.client = client;
      this.adapter = client.adapter;
      this.limits = client.limits;
      this.initMs = client.initMs;
      if (client.benchmark) this.recordBrowser(client.benchmark);
      return client;
    } catch (err) {
      if (this.choice !== 'server') this.failure = errorMessage(err);
      return null;
    } finally {
      this.starting = null;
      this.refresh();
    }
  }

  private release(): void {
    this.client?.dispose();
    this.client = null;
    this.adapter = null;
    this.limits = null;
    this.initMs = null;
    this.lastRenderMs = null;
  }

  private persist(): void {
    writeStored(STORAGE_KEY, {
      choice: this.choice,
      ...(this.browserMs === null ? {} : { browserMs: this.browserMs }),
      ...(this.serverMs === null ? {} : { serverMs: this.serverMs })
    });
  }

  private refresh(): void {
    const [state, reason] = this.status();
    this.state = state;
    this.reason = reason;
  }

  private status(): [RendererState, string | null] {
    if (this.choice === 'server') return ['server', SERVER_CHOSEN];
    if (this.failure) return ['server', this.failure];
    if (!this.client) return [this.starting ? 'starting' : 'idle', null];
    if (this.choice === 'browser') return ['browser', BROWSER_CHOSEN];
    const verdict = this.verdict();
    return [verdict.use ?? 'measuring', verdict.reason];
  }
}

export const renderer = new RendererStore();
