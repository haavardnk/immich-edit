import { toasts } from '#lib/stores/toasts.svelte.ts';
import { VIEWER_PATH, VIEWER_WINDOW } from '#lib/viewer/protocol.ts';

type ScreenArea = {
  availLeft: number;
  availTop: number;
  availWidth: number;
  availHeight: number;
};

type ScreenDetails = {
  screens: readonly ScreenArea[];
  currentScreen: ScreenArea;
};

type WindowManagement = {
  getScreenDetails?: () => Promise<ScreenDetails>;
};

type ExtendedScreen = { isExtended?: boolean };

async function otherScreen(): Promise<ScreenArea | null> {
  if (!(screen as ExtendedScreen).isExtended) return null;
  const details = (window as WindowManagement).getScreenDetails;
  if (!details) return null;
  const { screens, currentScreen } = await details.call(window);
  return screens.find((candidate) => candidate !== currentScreen) ?? null;
}

async function placeOnOtherScreen(viewer: Window): Promise<void> {
  try {
    const target = await otherScreen();
    if (!target) return;
    viewer.moveTo(target.availLeft, target.availTop);
    viewer.resizeTo(target.availWidth, target.availHeight);
  } catch {
    return;
  }
}

export function openViewer(): void {
  const viewer = window.open(VIEWER_PATH, VIEWER_WINDOW, 'popup,width=1280,height=800');
  if (!viewer) {
    toasts.push('error', 'Allow pop-ups to open the viewer window.');
    return;
  }
  viewer.focus();
  void placeOnOtherScreen(viewer);
}
