import { describe, expect, it } from 'vitest';
import { installPlatform, installState, type InstallInput } from '#lib/pwa/installState.ts';

const IPHONE =
  'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
const IPAD_DESKTOP =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15';
const MAC_CHROME =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36';
const MAC_FIREFOX =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:140.0) Gecko/20100101 Firefox/140.0';
const ANDROID =
  'Mozilla/5.0 (Linux; Android 15; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Mobile Safari/537.36';

const BASE: InstallInput = {
  standalone: false,
  secure: true,
  installed: false,
  canPrompt: false,
  platform: 'other'
};

describe('installPlatform', () => {
  it.each([
    ['an iPhone', IPHONE, 5, 'ios'],
    ['an iPad asking for the desktop site', IPAD_DESKTOP, 5, 'ios'],
    ['Safari on a Mac', IPAD_DESKTOP, 0, 'safari-mac'],
    ['Chrome on a Mac', MAC_CHROME, 0, 'other'],
    ['Firefox on a Mac', MAC_FIREFOX, 0, 'other'],
    ['Chrome on Android', ANDROID, 5, 'other']
  ])('detects %s', (_name, userAgent, touchPoints, platform) => {
    expect(installPlatform(userAgent, touchPoints)).toBe(platform);
  });
});

describe('installState', () => {
  it('hides the entry inside the installed app before anything else', () => {
    expect(installState({ ...BASE, standalone: true, secure: false, canPrompt: true })).toEqual({
      kind: 'hidden'
    });
  });

  it('explains HTTPS when the page is not a secure context', () => {
    expect(installState({ ...BASE, secure: false, canPrompt: true })).toEqual({
      kind: 'insecure'
    });
  });

  it('reports a finished install', () => {
    expect(installState({ ...BASE, installed: true, canPrompt: true })).toEqual({
      kind: 'installed'
    });
  });

  it('offers the browser prompt when one is stashed', () => {
    expect(installState({ ...BASE, canPrompt: true, platform: 'ios' })).toEqual({
      kind: 'prompt'
    });
  });

  it.each([
    ['ios', 'ios'],
    ['safari-mac', 'safari-mac'],
    ['other', 'browser']
  ] as const)('guides %s through a manual install', (platform, guide) => {
    expect(installState({ ...BASE, platform })).toEqual({ kind: 'guide', guide });
  });
});
