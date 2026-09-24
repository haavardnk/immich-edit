import { ClientRenderer } from '$lib/render/client-renderer';
import { errorMessage } from '$lib/utils/errors';
import { readStored, writeStored } from '$lib/utils/storage';

export type RendererChoice = 'auto' | 'server';
export type RendererState = 'idle' | 'starting' | 'browser' | 'server';

const STORAGE_KEY = 'immich-edit:renderer';
const SERVER_CHOSEN = 'Server rendering is selected';

class RendererStore {
  choice = $state<RendererChoice>('auto');
  state = $state<RendererState>('idle');
  reason = $state<string | null>(null);
  adapter = $state<string | null>(null);
  initMs = $state<number | null>(null);
  lastRenderMs = $state<number | null>(null);

  private client: ClientRenderer | null = null;
  private starting: Promise<ClientRenderer | null> | null = null;

  constructor() {
    if (readStored<{ choice: RendererChoice }>(STORAGE_KEY)?.choice === 'server') {
      this.choice = 'server';
    }
  }

  get current(): ClientRenderer | null {
    return this.client;
  }

  get undecided(): boolean {
    return this.choice === 'auto' && (this.state === 'idle' || this.state === 'starting');
  }

  start(): Promise<ClientRenderer | null> {
    if (this.choice === 'server') {
      this.useServer(SERVER_CHOSEN);
      return Promise.resolve(null);
    }
    if (this.client || this.state === 'server') return Promise.resolve(this.client);
    this.starting ??= this.launch();
    return this.starting;
  }

  setChoice(choice: RendererChoice): void {
    if (this.choice === choice) return;
    this.choice = choice;
    writeStored(STORAGE_KEY, { choice });
    this.release();
    if (choice === 'server') this.useServer(SERVER_CHOSEN);
    else this.useIdle();
  }

  fail(err: unknown): void {
    this.release();
    this.useServer(`The browser renderer failed: ${errorMessage(err)}`);
  }

  recordRender(ms: number): void {
    this.lastRenderMs = ms;
  }

  private async launch(): Promise<ClientRenderer | null> {
    this.state = 'starting';
    try {
      const client = await ClientRenderer.start();
      if (this.choice !== 'auto') {
        client.dispose();
        return null;
      }
      this.client = client;
      this.state = 'browser';
      this.reason = null;
      this.adapter = client.adapter;
      this.initMs = client.initMs;
      return client;
    } catch (err) {
      if (this.choice === 'auto') this.useServer(errorMessage(err));
      return null;
    } finally {
      this.starting = null;
    }
  }

  private release(): void {
    this.client?.dispose();
    this.client = null;
    this.adapter = null;
    this.initMs = null;
    this.lastRenderMs = null;
  }

  private useServer(reason: string): void {
    this.state = 'server';
    this.reason = reason;
  }

  private useIdle(): void {
    this.state = 'idle';
    this.reason = null;
  }
}

export const renderer = new RendererStore();
