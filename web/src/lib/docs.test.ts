import { describe, expect, it } from 'vitest';
import { docsUrl } from './docs.ts';

describe('docsUrl', () => {
  it('routes dev 0.0.0 builds to main', () => {
    expect(docsUrl('hosting/deployment/', '0.0.0')).toBe(
      'https://immich-edit.iridiumlab.dev/main/hosting/deployment/'
    );
    expect(docsUrl('', '0.0.0')).toBe('https://immich-edit.iridiumlab.dev/main/');
  });

  it('routes stamped release versions to vX.Y.Z', () => {
    expect(docsUrl('hosting/rendering/#browser-previews-on-a-local-network', '0.8.1')).toBe(
      'https://immich-edit.iridiumlab.dev/v0.8.1/hosting/rendering/#browser-previews-on-a-local-network'
    );
    expect(docsUrl('/hosting/troubleshooting/', 'v0.8.1-rc.1')).toBe(
      'https://immich-edit.iridiumlab.dev/v0.8.1-rc.1/hosting/troubleshooting/'
    );
  });

  it('uses the build-time __APP_VERSION__ by default', () => {
    expect(docsUrl('start/install/')).toBe(
      'https://immich-edit.iridiumlab.dev/main/start/install/'
    );
  });
});
