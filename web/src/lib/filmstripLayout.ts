import { assetAspectRatio, type GridAsset } from './assetGridLayout';

export type FilmstripBox = {
  left: number;
  width: number;
};

export type FilmstripLayout = {
  boxes: FilmstripBox[];
  width: number;
};

export function createFilmstripLayout(
  assets: GridAsset[],
  height: number,
  gap: number,
  padding: number
): FilmstripLayout {
  let left = padding;
  const boxes = assets.map((asset) => {
    const width = height * assetAspectRatio(asset);
    const box = { left, width };
    left += width + gap;
    return box;
  });
  const contentWidth = boxes.length > 0 ? left - gap : padding;
  return { boxes, width: contentWidth + padding };
}

export function visibleFilmstripRange(
  boxes: FilmstripBox[],
  left: number,
  right: number,
  overscan: number
): { startIndex: number; endIndex: number } {
  if (boxes.length === 0) return { startIndex: 0, endIndex: 0 };

  let startIndex = 0;
  for (const box of boxes) {
    if (box.left + box.width >= left) break;
    startIndex += 1;
  }
  startIndex = Math.max(0, startIndex - overscan);

  let endIndex = startIndex;
  while (endIndex < boxes.length && (boxes[endIndex]?.left ?? Infinity) <= right) endIndex += 1;
  endIndex = Math.min(boxes.length, endIndex + overscan);
  return { startIndex, endIndex };
}
