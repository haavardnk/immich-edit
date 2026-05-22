export type ToastKind = 'error' | 'warn' | 'info';

export type Toast = {
  id: number;
  kind: ToastKind;
  message: string;
};

class ToastStore {
  items = $state<Toast[]>([]);
  private nextId = 1;
  private recent = new Map<string, number>();

  push = (kind: ToastKind, message: string, ttlMs: number = 6000): void => {
    const key = `${kind}:${message}`;
    const now = Date.now();
    const last = this.recent.get(key);
    if (last != null && now - last < 3000) return;
    this.recent.set(key, now);
    const id = this.nextId++;
    this.items = [...this.items, { id, kind, message }];
    if (ttlMs > 0) setTimeout(() => this.dismiss(id), ttlMs);
  };

  dismiss = (id: number): void => {
    this.items = this.items.filter((t) => t.id !== id);
  };
}

export const toasts = new ToastStore();
