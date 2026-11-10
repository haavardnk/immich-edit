import { ConflictError } from '#lib/api/client.ts';
import { deleteEdits, putEdits, restoreEdits } from '#lib/api/edits.ts';
import { restoreSnapshot } from '#lib/api/snapshots.ts';
import { manifestToEdits } from '#lib/edits/manifest.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { isIdentity, neutralEdits, type EditRecord, type Edits } from '#lib/types/edits.ts';
import { errorMessage } from '#lib/utils/errors.ts';

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
  private queued = 0;

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

  settled(): Promise<void> {
    return this.tail;
  }

  commit(edits: Edits, action?: string): Promise<void> {
    const session = this.session;
    if (!session) return Promise.resolve();
    this.lastAction = action;
    this.queued++;
    return this.queueSave(session, edits, action);
  }

  retry(): Promise<void> {
    const session = this.session;
    if (!this.ctx.initialised || !session) return Promise.resolve();
    session.blocked = false;
    this.ctx.saveError = null;
    this.queued++;
    return this.queueSave(session, $state.snapshot(this.ctx.edits) as Edits, this.lastAction);
  }

  restoreEntry(entryId: number): Promise<void> {
    const session = this.session;
    if (!this.ctx.initialised || !session) return Promise.resolve();
    session.blocked = false;
    this.ctx.saveError = null;
    const ticket = ++this.queued;
    return this.queueTask(session, async () => {
      if (this.session !== session) return;
      const saved = await restoreEdits(session.assetId, entryId);
      if (this.session !== session) return;
      this.adopt(session, ticket, saved);
    });
  }

  restoreSnapshot(snapshotId: string): Promise<boolean> {
    const session = this.session;
    if (!this.ctx.initialised || !session) return Promise.resolve(false);
    session.blocked = false;
    this.ctx.saveError = null;
    let restored = false;
    const ticket = ++this.queued;
    return this.queueTask(session, async () => {
      if (this.session !== session) return;
      try {
        const saved = await restoreSnapshot(session.assetId, snapshotId, session.hash);
        session.hash = saved.hash;
        if (this.session !== session) return;
        this.adopt(session, ticket, saved);
        restored = true;
      } catch (e) {
        if (!(e instanceof ConflictError)) throw e;
        this.fail(session, e);
      }
    }).then(() => restored);
  }

  private adopt(session: SaveSession, ticket: number, saved: EditRecord | null): void {
    session.hash = saved?.hash ?? '';
    this.ctx.savedHash = session.hash;
    this.ctx.saveError = null;
    this.ctx.error = null;
    if (ticket !== this.queued) return;
    this.ctx.edits = saved ? manifestToEdits(saved.manifest) : neutralEdits();
    this.ctx.onLive();
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
      this.fail(session, e);
    }
  }

  private fail(session: SaveSession, e: unknown): void {
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
