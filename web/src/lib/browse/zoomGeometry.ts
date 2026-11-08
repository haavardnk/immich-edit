export type Box = { left: number; top: number; width: number; height: number };

export type Viewport = { width: number; height: number };

const MIN_VISIBLE_SHARE = 0.5;

export function containBox(frame: Box, ratio: number): Box {
  if (!(ratio > 0) || frame.width <= 0 || frame.height <= 0) return frame;
  const width = Math.min(frame.width, frame.height * ratio);
  const height = width / ratio;
  return {
    left: frame.left + (frame.width - width) / 2,
    top: frame.top + (frame.height - height) / 2,
    width,
    height
  };
}

export function onScreen(box: Box, viewport: Viewport): boolean {
  const area = box.width * box.height;
  if (area <= 0) return false;
  const width = Math.min(box.left + box.width, viewport.width) - Math.max(box.left, 0);
  const height = Math.min(box.top + box.height, viewport.height) - Math.max(box.top, 0);
  if (width <= 0 || height <= 0) return false;
  return (width * height) / area >= MIN_VISIBLE_SHARE;
}

export function boxFrame(box: Box): Keyframe {
  return {
    left: `${box.left}px`,
    top: `${box.top}px`,
    width: `${box.width}px`,
    height: `${box.height}px`
  };
}
