export type InstallPlatform = 'ios' | 'safari-mac' | 'other';

export type InstallGuide = 'ios' | 'safari-mac' | 'browser';

export type InstallState =
  | { kind: 'hidden' }
  | { kind: 'insecure' }
  | { kind: 'installed' }
  | { kind: 'prompt' }
  | { kind: 'guide'; guide: InstallGuide };

export type InstallInput = {
  standalone: boolean;
  secure: boolean;
  installed: boolean;
  canPrompt: boolean;
  platform: InstallPlatform;
};

const OTHER_MAC_BROWSERS = /Chrome|Chromium|CriOS|Edg|OPR|Firefox|FxiOS/;

export function installPlatform(userAgent: string, maxTouchPoints: number): InstallPlatform {
  if (/iPhone|iPad|iPod/.test(userAgent)) return 'ios';
  if (!userAgent.includes('Macintosh')) return 'other';
  if (maxTouchPoints > 0) return 'ios';
  if (userAgent.includes('Safari/') && !OTHER_MAC_BROWSERS.test(userAgent)) return 'safari-mac';
  return 'other';
}

function guideFor(platform: InstallPlatform): InstallGuide {
  return platform === 'other' ? 'browser' : platform;
}

export function installState(input: InstallInput): InstallState {
  if (input.standalone) return { kind: 'hidden' };
  if (!input.secure) return { kind: 'insecure' };
  if (input.installed) return { kind: 'installed' };
  if (input.canPrompt) return { kind: 'prompt' };
  return { kind: 'guide', guide: guideFor(input.platform) };
}
