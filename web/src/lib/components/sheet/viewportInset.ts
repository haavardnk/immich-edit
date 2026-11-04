export type ViewportMetrics = { innerHeight: number; height: number; offsetTop: number };

export type ViewportInset = { top: number; bottom: number };

export const NO_INSET: ViewportInset = { top: 0, bottom: 0 };

export function keyboardInset({ innerHeight, height, offsetTop }: ViewportMetrics): ViewportInset {
  const top = Math.max(0, Math.round(offsetTop));
  return { top, bottom: Math.max(0, Math.round(innerHeight - height - offsetTop)) };
}
