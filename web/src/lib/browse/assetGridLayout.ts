import createJustifiedLayout from 'justified-layout';
const FALLBACK_ASPECT_RATIO = 3 / 2;

export type GridAsset = {
  exifInfo: {
    exifImageWidth: number | null;
    exifImageHeight: number | null;
    orientation?: string | null;
  } | null;
};

export type AssetGridBox = {
  top: number;
  left: number;
  width: number;
  height: number;
  row: number;
  column: number;
};

export type AssetGridRow = {
  top: number;
  height: number;
  startIndex: number;
  endIndex: number;
};

export type AssetGridLayout = {
  boxes: AssetGridBox[];
  rows: AssetGridRow[];
  height: number;
};

export function assetAspectRatio(asset: GridAsset): number {
  const width = asset.exifInfo?.exifImageWidth;
  const height = asset.exifInfo?.exifImageHeight;
  if (!width || !height || width <= 0 || height <= 0) return FALLBACK_ASPECT_RATIO;
  const orientation = Number(asset.exifInfo?.orientation);
  if ([5, 6, 7, 8, 90, -90].includes(orientation)) return height / width;
  return width / height;
}

export function createAssetGridLayout(
  assets: GridAsset[],
  width: number,
  targetRowHeight: number,
  gap: number
): AssetGridLayout {
  if (assets.length === 0 || width <= 0) return { boxes: [], rows: [], height: 0 };

  const geometry = createJustifiedLayout(assets.map(assetAspectRatio), {
    containerWidth: width,
    containerPadding: 0,
    boxSpacing: gap,
    targetRowHeight,
    targetRowHeightTolerance: 0.5,
    widowLayoutStyle: 'left'
  });
  const rows: AssetGridRow[] = [];
  const boxes: AssetGridBox[] = geometry.boxes.map((box, index) => {
    let rowIndex = rows.length - 1;
    let row = rows[rowIndex];
    if (!row || row.top !== box.top) {
      rowIndex += 1;
      row = { top: box.top, height: box.height, startIndex: index, endIndex: index + 1 };
      rows.push(row);
    } else {
      row.endIndex = index + 1;
    }
    return { ...box, row: rowIndex, column: index - row.startIndex };
  });

  return { boxes, rows, height: geometry.containerHeight };
}

export function visibleAssetRange(
  layout: AssetGridLayout,
  top: number,
  bottom: number,
  overscanRows: number
): { startIndex: number; endIndex: number } {
  if (layout.rows.length === 0) return { startIndex: 0, endIndex: 0 };

  let low = 0;
  let high = layout.rows.length;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    const row = layout.rows[middle];
    if (row && row.top + row.height < top) low = middle + 1;
    else high = middle;
  }
  const startRow = Math.max(0, low - overscanRows);

  low = startRow;
  high = layout.rows.length;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    const row = layout.rows[middle];
    if (row && row.top <= bottom) low = middle + 1;
    else high = middle;
  }
  const endRow = Math.min(layout.rows.length, low + overscanRows);
  return {
    startIndex: layout.rows[startRow]?.startIndex ?? layout.boxes.length,
    endIndex: layout.rows[endRow - 1]?.endIndex ?? layout.boxes.length
  };
}

export function verticalAssetIndex(
  layout: AssetGridLayout,
  index: number,
  rowDelta: number
): number {
  const box = layout.boxes[index];
  if (!box) return 0;
  const targetRowIndex = Math.min(layout.rows.length - 1, Math.max(0, box.row + rowDelta));
  const targetRow = layout.rows[targetRowIndex];
  if (!targetRow) return index;
  const center = box.left + box.width / 2;
  let closestIndex = targetRow.startIndex;
  let closestDistance = Number.POSITIVE_INFINITY;
  for (let candidate = targetRow.startIndex; candidate < targetRow.endIndex; candidate += 1) {
    const candidateBox = layout.boxes[candidate];
    if (!candidateBox) continue;
    const distance = Math.abs(candidateBox.left + candidateBox.width / 2 - center);
    if (distance < closestDistance) {
      closestIndex = candidate;
      closestDistance = distance;
    }
  }
  return closestIndex;
}
