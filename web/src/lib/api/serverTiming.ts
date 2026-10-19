export interface ServerTiming {
  renderMs: number;
  encodeMs: number;
  transferMs: number;
}

export function parseServerTiming(header: string | null, transferMs: number): ServerTiming | null {
  if (!header) return null;
  const durations = new Map<string, number>();
  for (const metric of header.split(',')) {
    const [name, ...params] = metric.split(';').map((part) => part.trim());
    const dur = params.find((param) => param.startsWith('dur='));
    const ms = dur ? Number(dur.slice(4)) : Number.NaN;
    if (name && Number.isFinite(ms)) durations.set(name, ms);
  }
  const renderMs = durations.get('render');
  const encodeMs = durations.get('encode');
  if (renderMs === undefined || encodeMs === undefined) return null;
  return { renderMs, encodeMs, transferMs };
}
