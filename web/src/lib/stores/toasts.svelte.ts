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

export const TOAST_TTL_MS: Record<ToastKind, number> = {
  info: 3000,
  success: 3000,
  warn: 6000,
  error: 8000
};
const ACTION_TTL_MS = 8000;

function lifetime(kind: ToastKind, action: ToastAction | undefined): number {
  return action ? Math.max(ACTION_TTL_MS, TOAST_TTL_MS[kind]) : TOAST_TTL_MS[kind];
}

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
    const ttl = ttlMs ?? lifetime(kind, action);
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

  fail = (context: string, e: unknown): void => {
    this.push('error', `${context}: ${errorMessage(e)}`);
  };

  tally = (failed: number, failure: string, success: string): void => {
    if (failed > 0) this.push('warn', failure);
    else this.push('success', success);
  };
}

export const toasts = new ToastStore();
