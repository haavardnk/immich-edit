import { NEUTRAL_RECORD } from './helpers';

export const FULL_RETOUCH_RECORD = {
  ...NEUTRAL_RECORD,
  hash: 'hash-full-retouch',
  manifest: {
    schema_version: 3,
    ops: {
      retouch: {
        strokes: Array.from({ length: 64 }, (_, i) => ({
          id: `spot-${i}`,
          mode: i % 2 === 0 ? 'heal' : 'clone',
          points: [{ x: 0.05 + (i % 8) * 0.01, y: 0.05 + Math.floor(i / 8) * 0.01 }],
          radius: 0.005,
          hardness: 0.5,
          opacity: 1,
          source: { x: 0.2, y: 0.2 },
          enabled: true
        }))
      }
    }
  }
};
