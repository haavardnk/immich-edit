import { pinElement } from '#lib/map/pin.ts';
import type { LngLat, Map as MapLibreMap, Marker } from 'maplibre-gl';

const LONG_PRESS_MS = 450;
const MOUSE_HOLD_MS = 250;
const MOUSE_SLOP = 4;
const TOUCH_SLOP = 8;
const DRAG_LIFT = 40;
export const TOUCH_REACH = 24;

export interface MarkerDrop {
  to: LngLat;
  from: LngLat;
}

function liftedPin(): HTMLElement {
  const pin = pinElement();
  pin.dataset['dragPin'] = '';
  pin.className = 'pointer-events-none absolute top-full left-1/2 -translate-x-1/2';
  pin.style.height = `${DRAG_LIFT}px`;
  return pin;
}

function exceedsSlop(pointer: string, dx: number, dy: number): boolean {
  return Math.hypot(dx, dy) > (pointer === 'mouse' ? MOUSE_SLOP : TOUCH_SLOP);
}

export function attachMarkerDrag(
  map: MapLibreMap,
  marker: Marker,
  element: HTMLElement,
  anchor: 'bottom' | 'center',
  onDrop: (drop: MarkerDrop) => void
): () => void {
  let dragged = false;

  const swallowClick = (event: MouseEvent): void => {
    if (!dragged) return;
    dragged = false;
    event.stopImmediatePropagation();
    event.preventDefault();
  };

  const noMenu = (event: Event): void => event.preventDefault();

  const down = (event: PointerEvent): void => {
    if (event.button !== 0) return;
    const mouse = event.pointerType === 'mouse';
    const start = { x: event.clientX, y: event.clientY };
    const from = marker.getLngLat();
    let armed = false;
    let moved = false;
    const reach = mouse ? 0 : TOUCH_REACH;
    let pin: HTMLElement | null = null;

    const escape = (key: KeyboardEvent): void => {
      if (key.key !== 'Escape') return;
      key.preventDefault();
      key.stopPropagation();
      dragged = true;
      finish();
      marker.setLngLat(from);
    };

    const arm = (): void => {
      armed = true;
      window.addEventListener('keydown', escape, true);
      map.dragPan.disable();
      element.dataset['dragging'] = '';
      pin = liftedPin();
      element.append(pin);
      const centred = anchor === 'center' ? element.offsetHeight / 2 : 0;
      marker.setOffset([0, -(DRAG_LIFT + centred)]);
    };

    const finish = (): void => {
      clearTimeout(timer);
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      window.removeEventListener('pointercancel', cancel);
      window.removeEventListener('keydown', escape, true);
      if (!armed) return;
      map.dragPan.enable();
      delete element.dataset['dragging'];
      pin?.remove();
      marker.setOffset([0, 0]);
    };

    const move = (next: PointerEvent): void => {
      if (next.pointerId !== event.pointerId) return;
      const far = exceedsSlop(event.pointerType, next.clientX - start.x, next.clientY - start.y);
      if (!armed) {
        if (far) finish();
        return;
      }
      moved = true;
      next.preventDefault();
      const box = map.getCanvasContainer().getBoundingClientRect();
      marker.setLngLat(map.unproject([next.clientX - box.left, next.clientY - box.top - reach]));
    };

    const up = (next: PointerEvent): void => {
      if (next.pointerId !== event.pointerId) return;
      const drop = armed && moved;
      finish();
      if (!drop) return;
      dragged = true;
      onDrop({ to: marker.getLngLat(), from });
    };

    const cancel = (next: PointerEvent): void => {
      if (next.pointerId !== event.pointerId) return;
      finish();
      if (armed && moved) marker.setLngLat(from);
    };

    const timer = setTimeout(arm, mouse ? MOUSE_HOLD_MS : LONG_PRESS_MS);
    window.addEventListener('pointermove', move, { passive: false });
    window.addEventListener('pointerup', up);
    window.addEventListener('pointercancel', cancel);
  };

  element.style.touchAction = 'none';
  element.addEventListener('pointerdown', down);
  element.addEventListener('click', swallowClick, true);
  element.addEventListener('contextmenu', noMenu);
  return () => {
    element.removeEventListener('pointerdown', down);
    element.removeEventListener('click', swallowClick, true);
    element.removeEventListener('contextmenu', noMenu);
  };
}
