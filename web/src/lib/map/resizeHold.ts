type Box = Pick<DOMRect, 'left' | 'top' | 'width' | 'height'>;

export function centreShift(before: Box, after: Box): { x: number; y: number } {
  return {
    x: after.left + after.width / 2 - (before.left + before.width / 2),
    y: after.top + after.height / 2 - (before.top + before.height / 2)
  };
}
