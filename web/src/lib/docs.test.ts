import { describe, expect, it } from 'vitest';
import { docsUrl } from './docs.ts';

describe('docsUrl', () => {
  it('routes dev 0.0.0 builds to main', () => {
    expect(docsUrl('hosting/deployment/', '0.0.0')).toBe(
      'https://haavardnk.github.io/immich-edit/main/hosting/deployment/'
    );
    expect(docsUrl('', '0.0.0')).toBe('https://haavardnk.github.io/immich-edit/main/');
  });

  it('routes stamped release versions to vX.Y.Z', () => {
    expect(docsUrl('hosting/rendering/#browser-previews-on-a-local-network', '0.9.0')).toBe(
      'https://haavardnk.github.io/immich-edit/v0.9.0/hosting/rendering/#browser-previews-on-a-local-network'
    );
    expect(docsUrl('/hosting/troubleshooting/', 'v0.9.0-rc.1')).toBe(
      'https://haavardnk.github.io/immich-edit/v0.9.0-rc.1/hosting/troubleshooting/'
    );
  });

  it('uses the build-time __APP_VERSION__ by default', () => {
    expect(docsUrl('start/install/')).toBe(
      'https://haavardnk.github.io/immich-edit/main/start/install/'
    );
  });
});
