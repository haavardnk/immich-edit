import type { Attachment } from 'svelte/attachments';

export type BarsTrack = {
  source: unknown;
  anchor: number;
  height: number;
  hidden: boolean;
};

export type ScrollSample = {
  source: unknown;
  top: number;
  scrollHeight: number;
  clientHeight: number;
};

export const SHOWN_BARS: BarsTrack = { source: null, anchor: 0, height: 0, hidden: false };

const SLOP = 8;

export function trackBars(track: BarsTrack, sample: ScrollSample, slop = SLOP): BarsTrack {
  const { source, top, scrollHeight, clientHeight } = sample;
  const end = scrollHeight - clientHeight;
  if (end <= 0) return track;
  if (source !== track.source || clientHeight !== track.height) {
    return { source, anchor: top, height: clientHeight, hidden: track.hidden && top > 0 };
  }
  if (top <= 0) return { ...track, anchor: 0, hidden: false };
  if (top >= end - slop) return track;
  const delta = top - track.anchor;
  if (delta > slop) return { ...track, anchor: top, hidden: true };
  if (delta < -slop) return { ...track, anchor: top, hidden: false };
  if (track.hidden ? delta > 0 : delta < 0) return { ...track, anchor: top };
  return track;
}

export function hideBarsOnScroll(spec: {
  enabled: () => boolean;
  route: () => unknown;
  onChange: (hidden: boolean) => void;
}): Attachment<HTMLElement> {
  return (frame) => {
    spec.route();
    spec.onChange(false);
    if (!spec.enabled()) return;
    let track = SHOWN_BARS;
    const onScroll = (event: Event): void => {
      const source = event.target;
      if (!(source instanceof Element)) return;
      const next = trackBars(track, {
        source,
        top: source.scrollTop,
        scrollHeight: source.scrollHeight,
        clientHeight: source.clientHeight
      });
      if (next.hidden !== track.hidden) spec.onChange(next.hidden);
      track = next;
    };
    frame.addEventListener('scroll', onScroll, { capture: true, passive: true });
    return () => {
      frame.removeEventListener('scroll', onScroll, { capture: true });
      spec.onChange(false);
    };
  };
}
