import { viewerMessage } from '#lib/viewer/protocol.ts';
import { describe, expect, it } from 'vitest';

describe('viewerMessage', () => {
  it('accepts every protocol kind', () => {
    for (const kind of ['editor-hello', 'editor-bye', 'viewer-hello', 'viewer-bye']) {
      expect(viewerMessage({ kind })).toEqual({ kind });
    }
  });

  it('rejects foreign payloads', () => {
    expect(viewerMessage(null)).toBeNull();
    expect(viewerMessage('render')).toBeNull();
    expect(viewerMessage({ kind: 'other' })).toBeNull();
    expect(viewerMessage({ kind: 3 })).toBeNull();
  });
});
