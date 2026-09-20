import { ConflictError } from '$lib/api/client';
import { deleteEdits, putEdits, restoreEdits } from '$lib/api/edits';
import { manifestToEdits } from '$lib/edits/manifest';
import { toasts } from '$lib/stores/toasts.svelte';
import { isIdentity, neutralEdits, type EditRecord, type Edits } from '$lib/types/edits';
import { errorMessage } from '$lib/utils/errors';

interface SaveSession {
  assetId: string;
  hash: string;
  pending: number;
  blocked: boolean;
}

export interface SaveCtx {
  initialised: boolean;
  edits: Edits;
  saving: boolean;
  savedHash: string;
  saveError: string | null;
  error: string | null;
  onLive(): void;
}

export class SaveQueue {
  private ctx: SaveCtx;
  private session: SaveSession | null = null;
  private tail: Promise<void> = Promise.resolve();
  private lastAction: string | undefined = undefined;

  constructor(ctx: SaveCtx) {
    this.ctx = ctx;
  }

  get open(): boolean {
    return this.session !== null;
  }

  begin(assetId: string, hash: string): void {
    this.session = { assetId, hash, pending: 0, blocked: false };
  }

  end(): void {
    this.session = null;
  }

  commit(edits: Edits, action?: string): Promise<void> {
    const session = this.session;
    if (!session) return Promise.resolve();
    this.lastAction = action;
    return this.queueSave(session, edits, action);
  }

  retry(): Promise<void> {
    const session = this.session;
    if (!this.ctx.initialised || !session) return Promise.resolve();
    session.blocked = false;
    this.ctx.saveError = null;
    return this.queueSave(session, $state.snapshot(this.ctx.edits) as Edits, this.lastAction);
  }

  restoreEntry(entryId: number): Promise<void> {
    const session = this.session;
    if (!this.ctx.initialised || !session) return Promise.resolve();
    session.blocked = false;
    this.ctx.saveError = null;
    return this.queueTask(session, async () => {
      if (this.session !== session) return;
      const saved = await restoreEdits(session.assetId, entryId);
      if (this.session !== session) return;
      this.ctx.edits = saved ? manifestToEdits(saved.manifest) : neutralEdits();
      session.hash = saved?.hash ?? '';
      this.ctx.savedHash = session.hash;
      this.ctx.error = null;
      this.ctx.onLive();
    });
  }

  private queueSave(session: SaveSession, edits: Edits, action?: string): Promise<void> {
    return this.queueTask(session, () => this.persist(session, edits, action));
  }

  private queueTask(session: SaveSession, task: () => Promise<void>): Promise<void> {
    session.pending++;
    if (this.session === session) this.ctx.saving = true;
    const queued = this.tail.then(async () => {
      if (session.blocked) return;
      await task();
    });
    this.tail = queued.catch(() => undefined);
    return queued.finally(() => {
      session.pending--;
      if (this.session === session) this.ctx.saving = session.pending > 0;
    });
  }

  private async persist(session: SaveSession, edits: Edits, action?: string): Promise<void> {
    try {
      if (isIdentity(edits)) {
        await deleteEdits(session.assetId, action, session.hash);
        session.hash = '';
      } else {
        const saved = await putEdits(session.assetId, edits, session.hash, action);
        session.hash = saved.hash;
      }
      if (this.session === session) {
        this.ctx.savedHash = session.hash;
        this.ctx.saveError = null;
      }
    } catch (e) {
      session.blocked = true;
      if (this.session !== session) return;
      if (e instanceof ConflictError) {
        const current = e.current as EditRecord | undefined;
        if (current) session.hash = current.hash;
        this.ctx.savedHash = session.hash;
        this.ctx.saveError = 'Edits changed elsewhere. Local edits kept. Retry to save them.';
        toasts.push('warn', this.ctx.saveError);
      } else {
        this.ctx.saveError = errorMessage(e);
        this.ctx.error = this.ctx.saveError;
      }
    }
  }
}
