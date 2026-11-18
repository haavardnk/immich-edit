import { errorMessage } from '#lib/utils/errors.ts';

export type ToastKind = 'error' | 'warn' | 'info' | 'success';

export type ToastAction = {
  label: string;
  run: () => void;
};

export type ToastOptions = {
  ttlMs?: number;
  action?: ToastAction;
  description?: string;
  pinned?: boolean;
  onDismiss?: () => void;
};

export type Toast = {
  id: number;
  kind: ToastKind;
  message: string;
  action?: ToastAction;
  description?: string;
  pinned: boolean;
  onDismiss?: () => void;
};

const MAX_VISIBLE = 3;

const DEFAULT_TTL: Record<ToastKind, number> = {
  info: 5000,
  warn: 8000,
  error: 12000,
  success: 10000
};

function evictable(items: Toast[]): Toast | undefined {
  if (items.length <= MAX_VISIBLE) return undefined;
  return items.find((toast) => !toast.pinned);
}

export class ToastStore {
  items = $state<Toast[]>([]);
  private nextId = 1;
  private recent = new Map<string, number>();

  push = (kind: ToastKind, message: string, options: ToastOptions = {}): void => {
    const { ttlMs, action, description, pinned = false, onDismiss } = options;
    const key = `${kind}:${message}`;
    const now = Date.now();
    const last = this.recent.get(key);
    if (!action && last != null && now - last < 3000) return;
    this.recent.set(key, now);
    const id = this.nextId++;
    const ttl = ttlMs ?? DEFAULT_TTL[kind];
    this.items = [...this.items, { id, kind, message, action, description, pinned, onDismiss }];
    for (let evicted = evictable(this.items); evicted; evicted = evictable(this.items)) {
      this.dismiss(evicted.id);
    }
    if (ttl > 0) setTimeout(() => this.dismiss(id), ttl);
  };

  dismiss = (id: number): void => {
    const toast = this.items.find((t) => t.id === id);
    if (!toast) return;
    this.items = this.items.filter((t) => t.id !== id);
    toast.onDismiss?.();
  };

  fail = (context: string, e: unknown, ttlMs?: number): void => {
    this.push('error', `${context}: ${errorMessage(e)}`, { ttlMs });
  };

  tally = (failed: number, failure: string, success: string): void => {
    if (failed > 0) this.push('warn', failure, { ttlMs: 6000 });
    else this.push('success', success, { ttlMs: 4000 });
  };
}

export const toasts = new ToastStore();
