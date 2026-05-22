<script lang="ts">
  let {
    hue = $bindable(),
    sat = $bindable(),
    size = 96,
    onLive,
    onCommit
  }: {
    hue: number;
    sat: number;
    size?: number;
    onLive: () => void;
    onCommit: () => void;
  } = $props();

  let wheel = $state<HTMLDivElement | undefined>();
  let dragging = $state(false);

  const cx = $derived(size / 2);
  const cy = $derived(size / 2);
  const r = $derived(size / 2);

  const thumb = $derived.by(() => {
    const angle = ((hue - 90) * Math.PI) / 180;
    const radius = (Math.min(Math.max(sat, 0), 100) / 100) * r;
    return { x: cx + radius * Math.cos(angle), y: cy + radius * Math.sin(angle) };
  });

  function updateFromEvent(ev: PointerEvent): void {
    if (!wheel) return;
    const rect = wheel.getBoundingClientRect();
    const px = ev.clientX - rect.left - rect.width / 2;
    const py = ev.clientY - rect.top - rect.height / 2;
    const dist = Math.sqrt(px * px + py * py);
    const maxR = rect.width / 2;
    const newSat = Math.min(100, Math.round((dist / maxR) * 100));
    let angle = (Math.atan2(py, px) * 180) / Math.PI + 90;
    if (angle < 0) angle += 360;
    hue = Math.round(angle);
    sat = newSat;
    onLive();
  }

  function pointerDown(ev: PointerEvent): void {
    if (!wheel) return;
    dragging = true;
    wheel.setPointerCapture(ev.pointerId);
    updateFromEvent(ev);
  }

  function pointerMove(ev: PointerEvent): void {
    if (!dragging) return;
    updateFromEvent(ev);
  }

  function pointerUp(ev: PointerEvent): void {
    if (!dragging) return;
    dragging = false;
    if (wheel?.hasPointerCapture(ev.pointerId)) wheel.releasePointerCapture(ev.pointerId);
    onCommit();
  }

  function reset(): void {
    hue = 0;
    sat = 0;
    onCommit();
  }

  function key(ev: KeyboardEvent): void {
    let changed = false;
    if (ev.key === 'ArrowLeft') {
      hue = (hue - 1 + 360) % 360;
      changed = true;
    } else if (ev.key === 'ArrowRight') {
      hue = (hue + 1) % 360;
      changed = true;
    } else if (ev.key === 'ArrowUp') {
      sat = Math.min(100, sat + 1);
      changed = true;
    } else if (ev.key === 'ArrowDown') {
      sat = Math.max(0, sat - 1);
      changed = true;
    }
    if (changed) {
      ev.preventDefault();
      onLive();
      onCommit();
    }
  }
</script>

<div
  bind:this={wheel}
  class="hue-wheel"
  style:width="{size}px"
  style:height="{size}px"
  role="slider"
  tabindex="0"
  aria-label="Hue and saturation wheel"
  aria-valuenow={hue}
  aria-valuemin={0}
  aria-valuemax={360}
  onpointerdown={pointerDown}
  onpointermove={pointerMove}
  onpointerup={pointerUp}
  onpointercancel={pointerUp}
  ondblclick={reset}
  onkeydown={key}
>
  <div class="wheel-conic"></div>
  <div class="wheel-radial"></div>
  <div
    class="wheel-thumb"
    style:left="{thumb.x}px"
    style:top="{thumb.y}px"
  ></div>
</div>

<style>
  .hue-wheel {
    position: relative;
    border-radius: 50%;
    cursor: crosshair;
    user-select: none;
    touch-action: none;
    outline: none;
  }
  .hue-wheel:focus-visible {
    box-shadow: 0 0 0 2px rgb(var(--immich-dark-primary));
  }
  .wheel-conic {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: conic-gradient(
      from 0deg,
      #ff0000,
      #ffff00,
      #00ff00,
      #00ffff,
      #0000ff,
      #ff00ff,
      #ff0000
    );
  }
  .wheel-radial {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(128, 128, 128, 1) 0%, rgba(128, 128, 128, 0) 65%);
  }
  .wheel-thumb {
    position: absolute;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: white;
    border: 2px solid rgba(0, 0, 0, 0.6);
    transform: translate(-50%, -50%);
    pointer-events: none;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
  }
</style>
