import { goto } from '$app/navigation';
import { page } from '$app/state';
import { editor } from '$lib/stores/editor.svelte';
import { ui } from '$lib/stores/ui.svelte';
import { browsing } from '$lib/stores/browsing.svelte';
import { backToGrid } from '$lib/backToGrid';
import { createVirtualCopy } from '$lib/copies';
import { nextRatingFromKey } from '$lib/ratingShortcuts';
import { isKeybind, isRadioGroupTarget, isTypingTarget, matchKeybind } from '$lib/keybinds';
import { activeContexts } from '$lib/keybindContext';
import { editorHref } from '$lib/editorNavigation';

const RETOUCH_SIZE = { step: 0.005, min: 0.005, max: 0.3 };
const BRUSH_SIZE = { step: 0.01, min: 0.005, max: 0.5 };
const HARDNESS = { step: 0.1, min: 0, max: 1 };

export function stepBrush(
  current: number,
  key: string,
  { step, min, max }: { step: number; min: number; max: number }
): number {
  const delta = key === '[' || key === '{' ? -step : step;
  return Math.min(max, Math.max(min, current + delta));
}

function onEscape(e: KeyboardEvent): void {
  if (ui.closeMetadataPopovers()) {
    e.preventDefault();
    return;
  }
  if (isTypingTarget(e)) return;
  if (ui.keybindsHelpOpen) {
    e.preventDefault();
    ui.closeKeybindsHelp();
  } else if (ui.editorTab === 'geometry' && ui.perspectiveCorners) {
    e.preventDefault();
    ui.perspectiveCorners = false;
  } else if (ui.editorTab === 'geometry' && !ui.fullscreen) {
    e.preventDefault();
    ui.editorTab = 'develop';
  } else if (ui.editorTab === 'retouch' && editor.activeRetouchId) {
    e.preventDefault();
    editor.activeRetouchId = null;
  } else if (editor.activeMaskComponentId) {
    e.preventDefault();
    editor.setActiveMaskComponent(null);
  } else if (ui.fullscreen) {
    e.preventDefault();
    ui.toggleFullscreen();
  }
}

export function editorKeydown(e: KeyboardEvent, id: string): void {
  if (isKeybind(e, 'editorEscape')) {
    onEscape(e);
    return;
  }
  if (isTypingTarget(e)) return;
  if (isKeybind(e, 'help')) {
    e.preventDefault();
    ui.toggleKeybindsHelp();
    return;
  }
  if (ui.keybindsHelpOpen) return;
  if (isRadioGroupTarget(e)) return;

  const bind = matchKeybind(e, activeContexts());
  if (!bind || bind === 'maskDelete' || bind === 'maskClosePolygon') return;
  e.preventDefault();

  switch (bind) {
    case 'editorNav': {
      const target = e.key === 'ArrowLeft' ? browsing.prevOf(id) : browsing.nextOf(id);
      if (target)
        void goto(editorHref(target.id, page.url.searchParams.get('from')), { replaceState: true });
      return;
    }
    case 'backToGrid':
      void backToGrid(id, page.url.searchParams.get('from'));
      return;
    case 'undo':
      editor.undo();
      return;
    case 'redo':
      editor.redo();
      return;
    case 'zoomToggle':
      editor.zoomCycle();
      return;
    case 'toggleInfo':
      ui.togglePopover('exif');
      return;
    case 'toggleTags':
      ui.togglePopover('tags');
      return;
    case 'clipWarn':
      editor.toggleClipWarn();
      return;
    case 'beforeAfter':
      editor.toggleSplit();
      return;
    case 'holdOriginal':
      if (!editor.showingOriginal) {
        editor.showingOriginal = true;
        editor.showOriginal();
      }
      return;
    case 'togglePanels':
      ui.togglePanels();
      return;
    case 'toggleChrome':
      ui.toggleChrome();
      return;
    case 'fullscreen':
      ui.toggleFullscreen();
      return;
    case 'openDevelop':
      ui.openTab('develop');
      return;
    case 'openGeometry':
      ui.openTab('geometry');
      return;
    case 'openRetouch':
      ui.openTab('retouch');
      return;
    case 'openMasks':
      ui.openTab('masks');
      return;
    case 'openExport':
      ui.openTab('export');
      return;
    case 'createVirtualCopy':
      void createVirtualCopy(id, { returnPath: page.url.searchParams.get('from') });
      return;
    case 'perspective':
      if (ui.editorTab !== 'geometry') ui.openTab('geometry');
      ui.togglePerspectiveCorners();
      return;
    case 'resetEdits':
      void editor.onReset();
      return;
    case 'copyEdits':
      editor.copyEdits();
      return;
    case 'pasteEdits':
      void editor.pasteEdits();
      return;
    case 'favorite':
      void editor.toggleFavorite();
      return;
    case 'reject':
      void editor.toggleReject();
      return;
    case 'unflag':
      void editor.clearFlags();
      return;
    case 'rate': {
      const next = nextRatingFromKey(e.key, editor.asset?.exifInfo?.rating ?? null);
      if (next !== undefined) void editor.setRating(next);
      return;
    }
    case 'maskOverlay':
      editor.toggleMaskOverlay();
      return;
    case 'retouchHeal':
      editor.setRetouchMode('heal');
      return;
    case 'retouchClone':
      editor.setRetouchMode('clone');
      return;
    case 'retouchDelete':
      if (editor.activeRetouchId) void editor.removeRetouchStroke(editor.activeRetouchId);
      return;
    case 'brushSize':
      if (ui.editorTab === 'retouch') {
        editor.setRetouchTool({ size: stepBrush(editor.retouchTool.size, e.key, RETOUCH_SIZE) });
      } else {
        editor.setBrushTool({ size: stepBrush(editor.brushTool.size, e.key, BRUSH_SIZE) });
      }
      return;
    case 'brushHardness':
      if (ui.editorTab === 'retouch') {
        editor.setRetouchTool({
          hardness: stepBrush(editor.retouchTool.hardness, e.key, HARDNESS)
        });
      } else {
        editor.setBrushTool({ hardness: stepBrush(editor.brushTool.hardness, e.key, HARDNESS) });
      }
      return;
  }
}

export function editorKeyup(e: KeyboardEvent): void {
  if (isKeybind(e, 'holdOriginal') && editor.showingOriginal) {
    editor.showingOriginal = false;
    editor.onLive();
  }
}
