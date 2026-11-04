const PINCH_EVENTS = ['gesturestart', 'gesturechange'] as const;

function block(event: Event): void {
  event.preventDefault();
}

export function lockPageZoom(): () => void {
  for (const type of PINCH_EVENTS) document.addEventListener(type, block, { passive: false });
  return () => {
    for (const type of PINCH_EVENTS) document.removeEventListener(type, block);
  };
}
