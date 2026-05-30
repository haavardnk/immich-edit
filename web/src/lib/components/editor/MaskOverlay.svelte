<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { isFullCrop, type MaskComponent, type MaskComponentKind, type Vec2f } from '$lib/types/edits';
  import {
    lensWarpFromEdits,
    lensWarpActive,
    maskUvToSceneUv,
    sceneUvToMaskUv,
    type LensWarpParams
  } from '$lib/utils/lensWarp';
  import {
    displayUvToMaskUv,
    maskUvToDisplayUv,
    type GeometryTransform,
    type RotateQuarter
  } from '$lib/utils/geomTransform';

  let {
    img
  }: {
    img: HTMLImageElement | null;
  } = $props();

  let rectX = $state(0);
  let rectY = $state(0);
  let rectW = $state(0);
  let rectH = $state(0);

  function recompute(): void {
    if (!img) return;
    const parent = img.parentElement;
    if (!parent) return;
    const p = parent.getBoundingClientRect();
    const r = img.getBoundingClientRect();
    rectX = r.left - p.left;
    rectY = r.top - p.top;
    rectW = r.width;
    rectH = r.height;
  }

  $effect(() => {
    if (!img) return;
    recompute();
    const ro = new ResizeObserver(recompute);
    ro.observe(img);
    if (img.parentElement) ro.observe(img.parentElement);
    img.addEventListener('load', recompute);
    window.addEventListener('resize', recompute);
    return () => {
      ro.disconnect();
      img.removeEventListener('load', recompute);
      window.removeEventListener('resize', recompute);
    };
  });

  $effect(() => {
    void ui.zoom;
    void ui.panX;
    void ui.panY;
    if (!img) return;
    const id = requestAnimationFrame(recompute);
    return () => cancelAnimationFrame(id);
  });

  const geomIdentity = $derived.by(() => {
    const g = editor.edits.geometry;
    return (
      g.rotate === 0 &&
      g.rotate_angle === 0 &&
      !g.flip_h &&
      !g.flip_v &&
      isFullCrop(g.crop)
    );
  });

  const lensP = $derived.by<LensWarpParams>(() =>
    lensWarpFromEdits(
      editor.edits.lens,
      editor.meta?.source_w ?? 1,
      editor.meta?.source_h ?? 1
    )
  );

  const geomT = $derived.by<GeometryTransform>(() => {
    const g = editor.edits.geometry;
    const sw = editor.meta?.source_w ?? 1;
    const sh = editor.meta?.source_h ?? 1;
    const dw = editor.meta?.width ?? sw;
    const dh = editor.meta?.height ?? sh;
    return {
      inputW: sw,
      inputH: sh,
      rotateQuarter: g.rotate as RotateQuarter,
      flipH: g.flip_h,
      flipV: g.flip_v,
      angleDeg: g.rotate_angle,
      crop: g.crop ?? { x: 0, y: 0, w: 1, h: 1 },
      outputW: dw,
      outputH: dh
    };
  });

  const fillIdentity = $derived(geomIdentity && !lensWarpActive(lensP));

  const active = $derived(
    editor.activeLayerId
      ? editor.edits.masks.find((l) => l.id === editor.activeLayerId) ?? null
      : null
  );

  const showOverlay = $derived(
    editor.maskOverlayVisible &&
      !!active &&
      editor.maskPreviewLayerId === null &&
      rectW > 0 &&
      rectH > 0
  );

  function sceneToDisplay(scene: Vec2f): Vec2f {
    const m = sceneUvToMaskUv(lensP, [scene.x, scene.y]);
    const d = maskUvToDisplayUv(geomT, m);
    return { x: d[0], y: d[1] };
  }

  function displayToScene(disp: Vec2f): Vec2f {
    const m = displayUvToMaskUv(geomT, [disp.x, disp.y]);
    const s = maskUvToSceneUv(lensP, m);
    return { x: s[0], y: s[1] };
  }

  function sceneToPx(scene: Vec2f): { x: number; y: number } {
    return toPx(sceneToDisplay(scene));
  }

  function fromPxScene(px: number, py: number): Vec2f {
    return displayToScene(fromPx(px, py));
  }

  function toPx(v: Vec2f): { x: number; y: number } {
    return { x: rectX + v.x * rectW, y: rectY + v.y * rectH };
  }

  function fromPx(px: number, py: number): Vec2f {
    return {
      x: Math.max(0, Math.min(1, (px - rectX) / Math.max(rectW, 1))),
      y: Math.max(0, Math.min(1, (py - rectY) / Math.max(rectH, 1)))
    };
  }

  type DragKind =
    | { kind: 'linear-p0' }
    | { kind: 'linear-p1' }
    | { kind: 'linear-move'; startP0: Vec2f; startP1: Vec2f; downAtN: Vec2f }
    | { kind: 'linear-feather' }
    | { kind: 'radial-center' }
    | { kind: 'radial-rx'; sign: 1 | -1 }
    | { kind: 'radial-ry'; sign: 1 | -1 }
    | { kind: 'radial-feather' };

  let drag = $state<{
    layerId: string;
    componentId: string;
    kind: DragKind;
  } | null>(null);

  function startDrag(
    e: PointerEvent,
    layerId: string,
    componentId: string,
    kind: DragKind
  ): void {
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    editor.setActiveMaskComponent(componentId);
    drag = { layerId, componentId, kind };
  }

  function selectOnly(e: PointerEvent, componentId: string): void {
    e.stopPropagation();
    editor.setActiveMaskComponent(componentId);
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key !== 'Backspace' && e.key !== 'Delete') return;
    const t = e.target as HTMLElement | null;
    if (t) {
      const tag = t.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || t.isContentEditable) return;
    }
    if (!active || !editor.activeMaskComponentId) return;
    e.preventDefault();
    void editor.removeMaskComponent(active.id, editor.activeMaskComponentId);
  }

  $effect(() => {
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  function onPointerMove(e: PointerEvent): void {
    if (!drag || !active) return;
    const layer = editor.edits.masks.find((l) => l.id === drag!.layerId);
    if (!layer) return;
    const comp = layer.components.find((c) => c.id === drag!.componentId);
    if (!comp) return;
    const n = fromPxScene(e.clientX - parentLeft(e), e.clientY - parentTop(e));
    let next: MaskComponentKind | null = null;
    if (drag.kind.kind === 'linear-p0' && comp.kind.kind === 'linear') {
      next = { ...comp.kind, p0: n };
    } else if (drag.kind.kind === 'linear-p1' && comp.kind.kind === 'linear') {
      next = { ...comp.kind, p1: n };
    } else if (drag.kind.kind === 'linear-move' && comp.kind.kind === 'linear') {
      const dx = n.x - drag.kind.downAtN.x;
      const dy = n.y - drag.kind.downAtN.y;
      next = {
        ...comp.kind,
        p0: { x: clamp01(drag.kind.startP0.x + dx), y: clamp01(drag.kind.startP0.y + dy) },
        p1: { x: clamp01(drag.kind.startP1.x + dx), y: clamp01(drag.kind.startP1.y + dy) }
      };
    } else if (drag.kind.kind === 'radial-center' && comp.kind.kind === 'radial') {
      next = { ...comp.kind, center: n };
    } else if (drag.kind.kind === 'radial-rx' && comp.kind.kind === 'radial') {
      const rx = Math.max(0.005, Math.abs(n.x - comp.kind.center.x));
      next = { ...comp.kind, radius_xy: { x: rx, y: comp.kind.radius_xy.y } };
    } else if (drag.kind.kind === 'radial-ry' && comp.kind.kind === 'radial') {
      const ry = Math.max(0.005, Math.abs(n.y - comp.kind.center.y));
      next = { ...comp.kind, radius_xy: { x: comp.kind.radius_xy.x, y: ry } };
    } else if (drag.kind.kind === 'linear-feather' && comp.kind.kind === 'linear') {
      const p0 = comp.kind.p0;
      const p1 = comp.kind.p1;
      const dx = p1.x - p0.x;
      const dy = p1.y - p0.y;
      const len2 = Math.max(1e-9, dx * dx + dy * dy);
      const mx = (p0.x + p1.x) * 0.5;
      const my = (p0.y + p1.y) * 0.5;
      const t = ((n.x - mx) * dx + (n.y - my) * dy) / len2;
      const feather = clamp01(2 * Math.abs(t));
      next = { ...comp.kind, feather };
    } else if (drag.kind.kind === 'radial-feather' && comp.kind.kind === 'radial') {
      const c = comp.kind.center;
      const rxy = comp.kind.radius_xy;
      const ex = rxy.x < 1e-6 ? 0 : (n.x - c.x) / rxy.x;
      const ey = rxy.y < 1e-6 ? 0 : (n.y - c.y) / rxy.y;
      const d = Math.sqrt(ex * ex + ey * ey);
      const feather = clamp01(1 - d);
      next = { ...comp.kind, feather };
    }
    if (next) editor.updateMaskComponentKind(layer.id, comp.id, next, true);
  }

  function onPointerUp(e: PointerEvent): void {
    if (!drag) return;
    drag = null;
    (e.currentTarget as Element).releasePointerCapture?.(e.pointerId);
    void editor.commitMasks();
  }

  function parentLeft(e: PointerEvent): number {
    const svg = e.currentTarget as SVGSVGElement;
    return svg.getBoundingClientRect().left;
  }

  function parentTop(e: PointerEvent): number {
    const svg = e.currentTarget as SVGSVGElement;
    return svg.getBoundingClientRect().top;
  }

  function clamp01(v: number): number {
    return Math.max(0, Math.min(1, v));
  }

  function linearHandles(comp: MaskComponent, k: Extract<MaskComponentKind, { kind: 'linear' }>) {
    const a = sceneToPx(k.p0);
    const b = sceneToPx(k.p1);
    return { a, b, comp };
  }

  function radialHandles(comp: MaskComponent, k: Extract<MaskComponentKind, { kind: 'radial' }>) {
    const c = sceneToPx(k.center);
    const rxEnd = sceneToPx({ x: k.center.x + k.radius_xy.x, y: k.center.y });
    const ryEnd = sceneToPx({ x: k.center.x, y: k.center.y + k.radius_xy.y });
    const rxDx = rxEnd.x - c.x;
    const rxDy = rxEnd.y - c.y;
    const ryDx = ryEnd.x - c.x;
    const ryDy = ryEnd.y - c.y;
    const rx = Math.hypot(rxDx, rxDy);
    const ry = Math.hypot(ryDx, ryDy);
    const tilt = (Math.atan2(rxDy, rxDx) * 180) / Math.PI;
    return { c, rx, ry, tilt, rxDx, rxDy, ryDx, ryDy, comp };
  }

  const activeCompId = $derived(editor.activeMaskComponentId);
</script>

{#if showOverlay && active}
  <svg
    class="absolute inset-0 pointer-events-none"
    width="100%"
    height="100%"
    role="presentation"
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
  >
    {#each active.components as comp (comp.id)}
      {#if comp.enabled && activeCompId === comp.id && comp.kind.kind === 'linear'}
        {@const h = linearHandles(comp, comp.kind)}
        {@const isSel = activeCompId === comp.id}
        {@const k = comp.kind}
        {@const half = k.feather * 0.5}
        {@const dx = h.b.x - h.a.x}
        {@const dy = h.b.y - h.a.y}
        {@const len = Math.max(1, Math.hypot(dx, dy))}
        {@const ux = dx / len}
        {@const uy = dy / len}
        {@const px = -uy}
        {@const py = ux}
        {@const mx = (h.a.x + h.b.x) / 2}
        {@const my = (h.a.y + h.b.y) / 2}
        {@const lo = { x: mx - half * dx, y: my - half * dy }}
        {@const hi = { x: mx + half * dx, y: my + half * dy }}
        {@const guideLen = 60}
        {@const gradId = `mask-linear-${comp.id}`}
        {@const fillOp = (comp.invert ? 0.0 : 0.55) * comp.opacity}
        {@const emptyOp = (comp.invert ? 0.55 : 0.0) * comp.opacity}
        {@const stopLo = Math.max(0, 0.5 - half)}
        {@const stopHi = Math.min(1, 0.5 + half)}
        <g style="pointer-events: auto;">
          {#if isSel && fillIdentity}
            <defs>
              <linearGradient
                id={gradId}
                gradientUnits="userSpaceOnUse"
                x1={h.a.x}
                y1={h.a.y}
                x2={h.b.x}
                y2={h.b.y}
              >
                <stop offset="0" stop-color={active.color} stop-opacity={emptyOp} />
                <stop offset={stopLo} stop-color={active.color} stop-opacity={emptyOp} />
                <stop offset={stopHi} stop-color={active.color} stop-opacity={fillOp} />
                <stop offset="1" stop-color={active.color} stop-opacity={fillOp} />
              </linearGradient>
            </defs>
            <rect
              x={rectX}
              y={rectY}
              width={rectW}
              height={rectH}
              fill={`url(#${gradId})`}
              style="pointer-events: none;"
            />
          {/if}
          <line
            x1={h.a.x}
            y1={h.a.y}
            x2={h.b.x}
            y2={h.b.y}
            stroke={active.color}
            stroke-width="1.5"
            opacity="0.9"
            style="cursor: pointer;"
            role="button"
            aria-label="Select linear"
            tabindex="-1"
            onpointerdown={(e) => selectOnly(e, comp.id)}
          />
          <line
            x1={h.a.x}
            y1={h.a.y}
            x2={h.b.x}
            y2={h.b.y}
            stroke="black"
            stroke-width="0.5"
            opacity="0.5"
          />
          <circle
            cx={h.a.x}
            cy={h.a.y}
            r="8"
            fill={active.color}
            stroke="white"
            stroke-width="2"
            style="cursor: move;"
            role="button"
            aria-label="Linear start"
            tabindex="-1"
            onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'linear-p0' })}
          />
          <circle
            cx={h.b.x}
            cy={h.b.y}
            r="8"
            fill={active.color}
            stroke="white"
            stroke-width="2"
            style="cursor: move;"
            role="button"
            aria-label="Linear end"
            tabindex="-1"
            onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'linear-p1' })}
          />
          <circle
            cx={(h.a.x + h.b.x) / 2}
            cy={(h.a.y + h.b.y) / 2}
            r="6"
            fill="white"
            stroke={active.color}
            stroke-width="2"
            style="cursor: move;"
            role="button"
            aria-label="Move linear"
            tabindex="-1"
            onpointerdown={(e) => {
              if (comp.kind.kind !== 'linear') return;
              const svg = (e.currentTarget as SVGElement).ownerSVGElement!;
              const r = svg.getBoundingClientRect();
              const mid = fromPxScene(e.clientX - r.left, e.clientY - r.top);
              startDrag(e, active.id, comp.id, {
                kind: 'linear-move',
                startP0: { ...comp.kind.p0 },
                startP1: { ...comp.kind.p1 },
                downAtN: mid
              });
            }}
          />
          {#if isSel}
            <line
              x1={lo.x - px * guideLen}
              y1={lo.y - py * guideLen}
              x2={lo.x + px * guideLen}
              y2={lo.y + py * guideLen}
              stroke={active.color}
              stroke-width="1"
              stroke-dasharray="4 4"
              opacity="0.7"
            />
            <line
              x1={hi.x - px * guideLen}
              y1={hi.y - py * guideLen}
              x2={hi.x + px * guideLen}
              y2={hi.y + py * guideLen}
              stroke={active.color}
              stroke-width="1"
              stroke-dasharray="4 4"
              opacity="0.7"
            />
            <circle
              cx={hi.x}
              cy={hi.y}
              r="5"
              fill={active.color}
              fill-opacity="0.3"
              stroke={active.color}
              stroke-width="1.5"
              style="cursor: move;"
              role="button"
              aria-label="Linear feather"
              tabindex="-1"
              onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'linear-feather' })}
            />
            <circle
              cx={lo.x}
              cy={lo.y}
              r="5"
              fill={active.color}
              fill-opacity="0.3"
              stroke={active.color}
              stroke-width="1.5"
              style="cursor: move;"
              role="button"
              aria-label="Linear feather"
              tabindex="-1"
              onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'linear-feather' })}
            />
          {/if}
        </g>
      {:else if comp.enabled && activeCompId === comp.id && comp.kind.kind === 'radial'}
        {@const h = radialHandles(comp, comp.kind)}
        {@const isSel = activeCompId === comp.id}
        {@const innerScale = 1 - comp.kind.feather}
        {@const gradId = `mask-radial-${comp.id}`}
        {@const fillOp = (comp.invert ? 0.0 : 0.55) * comp.opacity}
        {@const emptyOp = (comp.invert ? 0.55 : 0.0) * comp.opacity}
        {@const rMax = Math.max(h.rx, h.ry, 1)}
        <g style="pointer-events: auto;">
          {#if isSel && fillIdentity}
            <defs>
              <radialGradient
                id={gradId}
                gradientUnits="userSpaceOnUse"
                cx={h.c.x}
                cy={h.c.y}
                r={rMax}
                gradientTransform={`translate(${h.c.x} ${h.c.y}) rotate(${h.tilt}) scale(${h.rx / rMax} ${h.ry / rMax}) translate(${-h.c.x} ${-h.c.y})`}
              >
                <stop offset="0" stop-color={active.color} stop-opacity={fillOp} />
                <stop offset={innerScale} stop-color={active.color} stop-opacity={fillOp} />
                <stop offset="1" stop-color={active.color} stop-opacity={emptyOp} />
              </radialGradient>
            </defs>
            <rect
              x={rectX}
              y={rectY}
              width={rectW}
              height={rectH}
              fill={`url(#${gradId})`}
              style="pointer-events: none;"
            />
          {/if}
          <ellipse
            cx={h.c.x}
            cy={h.c.y}
            rx={h.rx}
            ry={h.ry}
            transform={`rotate(${h.tilt} ${h.c.x} ${h.c.y})`}
            fill="none"
            stroke={active.color}
            stroke-width="1.5"
            opacity="0.9"
            style="cursor: pointer;"
            role="button"
            aria-label="Select radial"
            tabindex="-1"
            onpointerdown={(e) => selectOnly(e, comp.id)}
          />
          <ellipse
            cx={h.c.x}
            cy={h.c.y}
            rx={h.rx}
            ry={h.ry}
            transform={`rotate(${h.tilt} ${h.c.x} ${h.c.y})`}
            fill="none"
            stroke="black"
            stroke-width="0.5"
            opacity="0.5"
          />
          <circle
            cx={h.c.x}
            cy={h.c.y}
            r="6"
            fill="white"
            stroke={active.color}
            stroke-width="2"
            style="cursor: move;"
            role="button"
            aria-label="Radial center"
            tabindex="-1"
            onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'radial-center' })}
          />
          <circle
            cx={h.c.x + h.rxDx}
            cy={h.c.y + h.rxDy}
            r="6"
            fill={active.color}
            stroke="white"
            stroke-width="2"
            style="cursor: ew-resize;"
            role="button"
            aria-label="Radial radius x"
            tabindex="-1"
            onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'radial-rx', sign: 1 })}
          />
          <circle
            cx={h.c.x - h.rxDx}
            cy={h.c.y - h.rxDy}
            r="6"
            fill={active.color}
            stroke="white"
            stroke-width="2"
            style="cursor: ew-resize;"
            role="button"
            aria-label="Radial radius x"
            tabindex="-1"
            onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'radial-rx', sign: -1 })}
          />
          <circle
            cx={h.c.x + h.ryDx}
            cy={h.c.y + h.ryDy}
            r="6"
            fill={active.color}
            stroke="white"
            stroke-width="2"
            style="cursor: ns-resize;"
            role="button"
            aria-label="Radial radius y"
            tabindex="-1"
            onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'radial-ry', sign: 1 })}
          />
          <circle
            cx={h.c.x - h.ryDx}
            cy={h.c.y - h.ryDy}
            r="6"
            fill={active.color}
            stroke="white"
            stroke-width="2"
            style="cursor: ns-resize;"
            role="button"
            aria-label="Radial radius y"
            tabindex="-1"
            onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'radial-ry', sign: -1 })}
          />
          {#if isSel && innerScale > 0.001}
            <ellipse
              cx={h.c.x}
              cy={h.c.y}
              rx={h.rx * innerScale}
              ry={h.ry * innerScale}
              transform={`rotate(${h.tilt} ${h.c.x} ${h.c.y})`}
              fill="none"
              stroke={active.color}
              stroke-width="1"
              stroke-dasharray="4 4"
              opacity="0.7"
            />
            <circle
              cx={h.c.x + h.rxDx * innerScale}
              cy={h.c.y + h.rxDy * innerScale}
              r="5"
              fill={active.color}
              fill-opacity="0.3"
              stroke={active.color}
              stroke-width="1.5"
              style="cursor: move;"
              role="button"
              aria-label="Radial feather"
              tabindex="-1"
              onpointerdown={(e) => startDrag(e, active.id, comp.id, { kind: 'radial-feather' })}
            />
          {/if}
        </g>
      {/if}
    {/each}
  </svg>
{/if}
