import { leftLabel, paneLabels } from '#lib/components/editor/compare/compareText.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { compareTarget } from '#lib/stores/editor/compareLeft.svelte.ts';
import { displayedEdits } from '#lib/stores/editor/displayedEdits.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';
import {
  VIEWER_CHANNEL,
  viewerMessage,
  type ViewerCompare,
  type ViewerMessage,
  type ViewerRender,
  type ViewerView
} from '#lib/viewer/protocol.ts';

function viewerCompare(): ViewerCompare | null {
  const mode = editor.compare.mode;
  if (mode === 'off') return null;
  const left = compareTarget(editor);
  if (!left) return null;
  const labels = mode === 'split' ? paneLabels() : { left: leftLabel(), right: 'Current' };
  return { mode, left, labels };
}

function viewerRender(): ViewerRender | null {
  const assetId = editor.assetId;
  if (!assetId || !editor.initialised) return null;
  return {
    assetId,
    name: editor.asset?.originalFileName ?? '',
    edits: displayedEdits(editor, $state.snapshot(editor.edits) as Edits),
    dragging: editor.dragging,
    proofSpace: editor.proofSpace,
    gamutWarn: editor.gamutWarn,
    clipWarn: ui.clipWarn,
    compare: viewerCompare()
  };
}

function viewerView(): ViewerView {
  return {
    view: ui.view,
    srcLong: editor.sourceLong,
    split: editor.compare.splitPos,
    greyCanvas: ui.greyCanvas
  };
}

export function publishToViewer(): void {
  let channel = $state.raw<BroadcastChannel | null>(null);
  let listening = $state(0);

  function post(message: ViewerMessage): void {
    channel?.postMessage(message);
  }

  $effect(() => {
    if (typeof BroadcastChannel === 'undefined') return;
    const open = new BroadcastChannel(VIEWER_CHANNEL);
    const bye = (): void => open.postMessage({ kind: 'editor-bye' });
    open.onmessage = (event) => {
      const message = viewerMessage(event.data);
      if (message?.kind === 'viewer-hello') listening += 1;
      if (message?.kind !== 'viewer-bye') return;
      listening = 0;
      open.postMessage({ kind: 'editor-hello' });
    };
    channel = open;
    open.postMessage({ kind: 'editor-hello' });
    window.addEventListener('pagehide', bye);
    return () => {
      window.removeEventListener('pagehide', bye);
      bye();
      open.close();
      channel = null;
      listening = 0;
    };
  });

  $effect(() => {
    if (listening === 0) return;
    const render = viewerRender();
    if (render) post({ kind: 'render', render });
  });

  $effect(() => {
    if (listening === 0) return;
    post({ kind: 'view', view: viewerView() });
  });
}
