export type Roi = [number, number, number, number];

export type Rect = { left: number; top: number; width: number; height: number };

export type RenderRequest = {
  roi: Roi;
  maxEdge: number;
  fullEdge: number;
};

export type RenderRequestInput = {
  frame: Rect;
  visible: Roi;
  dpr: number;
  srcLong: number;
  serverMaxEdge: number;
  haveRoi: Roi | null;
  haveFullEdge: number;
};

const ROI_QUANT = 512;
const ROI_PAD = 0.1;
const REUSE_SLACK = 0.95;
const PLACE_TOLERANCE_PX = 2;

function clamp01(v: number): number {
  return Math.min(1, Math.max(0, v));
}

function safeDpr(dpr: number): number {
  return Number.isFinite(dpr) && dpr > 0 ? dpr : 1;
}

export function snapToDevice(v: number, dpr: number): number {
  return Math.round(v * safeDpr(dpr)) / safeDpr(dpr);
}

export function frameBox(
  containerW: number,
  containerH: number,
  srcW: number,
  srcH: number,
  zoom: number,
  panX: number,
  panY: number,
  dpr: number
): Rect | null {
  if (containerW <= 0 || containerH <= 0 || srcW <= 0 || srcH <= 0) return null;
  if (!Number.isFinite(zoom) || zoom <= 0) return null;
  const fit = Math.min(containerW / srcW, containerH / srcH);
  const scale = fit * (zoom / 100);
  const width = snapToDevice(srcW * scale, dpr);
  const height = snapToDevice(srcH * scale, dpr);
  return {
    left: snapToDevice((containerW - width) / 2 + panX, dpr),
    top: snapToDevice((containerH - height) / 2 + panY, dpr),
    width,
    height
  };
}

export function visibleRegion(frame: Rect, containerW: number, containerH: number): Roi | null {
  if (frame.width <= 0 || frame.height <= 0) return null;
  const x0 = clamp01(-frame.left / frame.width);
  const x1 = clamp01((containerW - frame.left) / frame.width);
  const y0 = clamp01(-frame.top / frame.height);
  const y1 = clamp01((containerH - frame.top) / frame.height);
  if (x1 <= x0 || y1 <= y0) return null;
  return [x0, y0, x1 - x0, y1 - y0];
}

export function roiCovers(have: Roi, visible: Roi): boolean {
  return (
    have[0] <= visible[0] &&
    have[1] <= visible[1] &&
    have[0] + have[2] >= visible[0] + visible[2] &&
    have[1] + have[3] >= visible[1] + visible[3]
  );
}

export function isFullFrame(roi: Roi): boolean {
  return roi[0] <= 0 && roi[1] <= 0 && roi[2] >= 1 && roi[3] >= 1;
}

export function renderRequest(input: RenderRequestInput): RenderRequest | null {
  const { frame, visible, srcLong, serverMaxEdge } = input;
  if (frame.width <= 0 || frame.height <= 0 || srcLong <= 0) return null;
  const dpr = safeDpr(input.dpr);
  const padX = visible[2] * ROI_PAD;
  const padY = visible[3] * ROI_PAD;
  const x0 = Math.floor(clamp01(visible[0] - padX) * ROI_QUANT) / ROI_QUANT;
  const x1 = Math.ceil(clamp01(visible[0] + visible[2] + padX) * ROI_QUANT) / ROI_QUANT;
  const y0 = Math.floor(clamp01(visible[1] - padY) * ROI_QUANT) / ROI_QUANT;
  const y1 = Math.ceil(clamp01(visible[1] + visible[3] + padY) * ROI_QUANT) / ROI_QUANT;
  const roi: Roi = [x0, y0, x1 - x0, y1 - y0];
  const longCss = Math.max(roi[2] * frame.width, roi[3] * frame.height);
  const longFraction = longCss / Math.max(frame.width, frame.height);
  const longNative = longFraction * srcLong;
  const wanted = Math.ceil(longCss * dpr);
  const maxEdge = Math.max(1, Math.min(wanted, Math.round(longNative), serverMaxEdge));
  const fullEdge = Math.round(maxEdge / longFraction);
  if (
    input.haveRoi &&
    input.haveFullEdge >= fullEdge * REUSE_SLACK &&
    roiCovers(input.haveRoi, visible)
  ) {
    return null;
  }
  return { roi, maxEdge, fullEdge };
}

export function placement(
  roi: Roi,
  frame: Rect,
  naturalW: number,
  naturalH: number,
  dpr: number
): Rect | null {
  if (naturalW <= 0 || naturalH <= 0 || frame.width <= 0) return null;
  const scale = safeDpr(dpr);
  const targetW = snapToDevice(roi[2] * frame.width, dpr);
  const targetH = snapToDevice(roi[3] * frame.height, dpr);
  const nativeW = naturalW / scale;
  const nativeH = naturalH / scale;
  const tolerance = PLACE_TOLERANCE_PX / scale;
  const exact =
    Math.abs(nativeW - targetW) <= tolerance && Math.abs(nativeH - targetH) <= tolerance;
  return {
    left: snapToDevice(frame.left + roi[0] * frame.width, dpr),
    top: snapToDevice(frame.top + roi[1] * frame.height, dpr),
    width: exact ? nativeW : targetW,
    height: exact ? nativeH : targetH
  };
}

export function zoomAnchor(
  containerW: number,
  containerH: number,
  frame: Rect,
  anchorX: number,
  anchorY: number,
  nextZoomRatio: number
): { panX: number; panY: number } {
  const nextW = frame.width * nextZoomRatio;
  const nextH = frame.height * nextZoomRatio;
  const u = frame.width > 0 ? (anchorX - frame.left) / frame.width : 0.5;
  const v = frame.height > 0 ? (anchorY - frame.top) / frame.height : 0.5;
  return {
    panX: anchorX - u * nextW - (containerW - nextW) / 2,
    panY: anchorY - v * nextH - (containerH - nextH) / 2
  };
}
