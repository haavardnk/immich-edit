import type { ColorSpaceOpt } from '#lib/api/export.ts';
import type { CompareTarget } from '#lib/stores/editor/compareLeft.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';
import type { View } from '#lib/utils/imageViewport.ts';

export const VIEWER_CHANNEL = 'immich-edit:viewer';
export const VIEWER_PATH = '/viewer';
export const VIEWER_WINDOW = 'immich-edit-viewer';

export type ViewerLabels = { left: string; right: string };

export type ViewerCompare = {
  mode: 'split' | 'side';
  left: CompareTarget;
  labels: ViewerLabels;
};

export type ViewerRender = {
  assetId: string;
  name: string;
  edits: Edits;
  dragging: boolean;
  proofSpace: ColorSpaceOpt;
  gamutWarn: boolean;
  clipWarn: boolean;
  compare: ViewerCompare | null;
};

export type ViewerView = {
  view: View;
  srcLong: number;
  split: number;
  greyCanvas: boolean;
};

export type ViewerMessage =
  | { kind: 'editor-hello' }
  | { kind: 'editor-bye' }
  | { kind: 'render'; render: ViewerRender }
  | { kind: 'view'; view: ViewerView }
  | { kind: 'viewer-hello' }
  | { kind: 'viewer-bye' };

const KINDS: ReadonlySet<string> = new Set([
  'editor-hello',
  'editor-bye',
  'render',
  'view',
  'viewer-hello',
  'viewer-bye'
]);

export function viewerMessage(data: unknown): ViewerMessage | null {
  if (typeof data !== 'object' || data === null) return null;
  const kind: unknown = Reflect.get(data, 'kind');
  return typeof kind === 'string' && KINDS.has(kind) ? (data as ViewerMessage) : null;
}
