<script lang="ts">
  import { gestures } from '#lib/gestures/gestures.ts';
  import { panRecognizer } from '#lib/gestures/pan.ts';

  let {
    at,
    origin,
    onstart,
    onmove,
    onend,
    testid = 'focus-ruler'
  }: {
    at: number;
    origin: number;
    onstart: () => void;
    onmove: (stepX: number, width: number) => void;
    onend: () => void;
    testid?: string;
  } = $props();

  let ruler = $state<HTMLDivElement | null>(null);
  let active = false;

  function stop(): void {
    if (!active) return;
    active = false;
    onend();
  }

  const drag = gestures({
    recognizers: [
      panRecognizer({
        lockAxis: true,
        thresholds: { panSlop: 8 },
        start: (pan) => {
          if (pan.axis !== 'x') return;
          active = true;
          onstart();
        },
        move: (pan) => {
          if (active) onmove(-pan.stepX, (ruler?.clientWidth ?? 0) * 2);
        },
        end: stop,
        cancel: stop
      })
    ]
  });
</script>

<div
  bind:this={ruler}
  aria-hidden="true"
  data-testid={testid}
  class="relative h-12 overflow-hidden text-dark/40"
  {@attach drag}
>
  <div
    class="absolute inset-y-3 left-1/2 w-[200%] bg-[repeating-linear-gradient(to_right,currentColor_0_1px,transparent_1px_12px)]"
    style:transform="translateX(-{at * 100}%)"
  >
    <span
      class="absolute -top-2.5 h-[calc(100%+0.625rem)] w-px bg-light-950/70"
      style:left="{origin * 100}%"
    ></span>
  </div>
  <span class="absolute inset-y-1 left-1/2 w-0.5 -translate-x-1/2 rounded-full bg-primary"></span>
</div>
