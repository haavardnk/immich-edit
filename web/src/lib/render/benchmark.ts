import type { Renderer } from './dispatch';
import type { TickSample } from './protocol';

const WARM_RUNS = 1;
const TIMED_RUNS = 3;

export const BENCHMARK_EDITS = JSON.stringify({
  basic: { exposure_ev: 0.4, contrast: 12, vibrance: 15, clarity: 10 },
  tone: { highlights: -35, shadows: 30, whites: 8, blacks: -6 }
});

interface Frame {
  bitmap?: ImageBitmap;
  width: number;
  height: number;
}

export async function runBenchmark(
  renderer: Renderer,
  edge: number,
  now: () => number = () => performance.now()
): Promise<TickSample> {
  const view = JSON.stringify({ max_edge: edge, histogram: true });
  renderer.set_synthetic_source(edge);
  try {
    const times: number[] = [];
    let pixels = 0;
    for (let run = 0; run < WARM_RUNS + TIMED_RUNS; run++) {
      const started = now();
      const frame = (await renderer.render(BENCHMARK_EDITS, view)) as Frame;
      const ms = now() - started;
      frame.bitmap?.close();
      if (run < WARM_RUNS) continue;
      times.push(ms);
      pixels = frame.width * frame.height;
    }
    return { ms: median(times), pixels };
  } finally {
    renderer.drop_source();
  }
}

function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)] ?? 0;
}
