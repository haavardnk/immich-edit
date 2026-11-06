import {
  createAssetGridLayout,
  type AssetGridBox,
  type AssetGridLayout,
  type AssetGridRow,
  type GridAsset
} from '#lib/browse/assetGridLayout.ts';

export type DaySection = { start: number; end: number };

export type DayHeaderBox = {
  top: number;
  height: number;
  section: number;
  start: number;
  end: number;
};

export type DayGridLayout = AssetGridLayout & { headers: DayHeaderBox[] };

export type DayGridOptions = {
  width: number;
  rowHeight: number;
  gap: number;
  headerHeight: number;
};

export function createDayGridLayout(
  assets: readonly GridAsset[],
  sections: readonly DaySection[] | null,
  { width, rowHeight, gap, headerHeight }: DayGridOptions
): DayGridLayout {
  if (!sections)
    return { ...createAssetGridLayout([...assets], width, rowHeight, gap), headers: [] };
  if (width <= 0 || assets.length === 0) return { boxes: [], rows: [], height: 0, headers: [] };
  const boxes: AssetGridBox[] = [];
  const rows: AssetGridRow[] = [];
  const headers: DayHeaderBox[] = [];
  let top = 0;
  sections.forEach(({ start, end }, section) => {
    if (end <= start) return;
    if (headers.length > 0) top += gap;
    headers.push({ top, height: headerHeight, section, start, end });
    top += headerHeight;
    const part = createAssetGridLayout(assets.slice(start, end), width, rowHeight, gap);
    const rowBase = rows.length;
    for (const row of part.rows) {
      rows.push({
        top: row.top + top,
        height: row.height,
        startIndex: row.startIndex + start,
        endIndex: row.endIndex + start
      });
    }
    for (const box of part.boxes)
      boxes.push({ ...box, top: box.top + top, row: box.row + rowBase });
    top += part.height;
  });
  return { boxes, rows, height: top, headers };
}

export function visibleHeaders(
  headers: readonly DayHeaderBox[],
  startIndex: number,
  endIndex: number
): DayHeaderBox[] {
  let low = 0;
  let high = headers.length;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    const header = headers[middle];
    if (header && header.end <= startIndex) low = middle + 1;
    else high = middle;
  }
  const shown: DayHeaderBox[] = [];
  for (let index = low; index < headers.length; index += 1) {
    const header = headers[index];
    if (!header || header.start >= endIndex) break;
    shown.push(header);
  }
  return shown;
}

export function sectionTop(layout: DayGridLayout, index: number): number | null {
  const box = layout.boxes[index];
  if (!box) return null;
  const header = layout.headers.find((candidate) => candidate.start === index);
  return header ? header.top : box.top;
}
