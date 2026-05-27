export type ToastKind = 'error' | 'warn' | 'info' | 'success';

export type Toast = {
  id: number;
  kind: ToastKind;
  message: string;
};

const MAX_VISIBLE = 3;

const DEFAULT_TTL: Record<ToastKind, number> = {
  info: 5000,
  warn: 8000,
  error: 12000,
  success: 10000
};

class ToastStore {
  items = $state<Toast[]>([]);
  private nextId = 1;
  private recent = new Map<string, number>();

  push = (kind: ToastKind, message: string, ttlMs?: number): void => {
    const key = `${kind}:${message}`;
    const now = Date.now();
    const last = this.recent.get(key);
    if (last != null && now - last < 3000) return;
    this.recent.set(key, now);
    const id = this.nextId++;
    const ttl = ttlMs ?? DEFAULT_TTL[kind];
    let next = [...this.items, { id, kind, message }];
    if (next.length > MAX_VISIBLE) next = next.slice(next.length - MAX_VISIBLE);
    this.items = next;
    if (ttl > 0) setTimeout(() => this.dismiss(id), ttl);
  };

  dismiss = (id: number): void => {
    this.items = this.items.filter((t) => t.id !== id);
  };
}

export const toasts = new ToastStore();
