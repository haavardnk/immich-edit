export const SETTLE_FALLBACK_MS = 450;

function seconds(value: string): number {
  const text = value.trim();
  const amount = Number.parseFloat(text);
  if (!Number.isFinite(amount)) return 0;
  return text.endsWith('ms') ? amount / 1000 : amount;
}

export function transitionMs(durations: string, delays: string): number {
  const delayList = delays.split(',');
  const longest = durations
    .split(',')
    .reduce(
      (most, duration, index) =>
        Math.max(most, seconds(duration) + seconds(delayList[index] ?? delayList[0] ?? '0s')),
      0
    );
  return longest * 1000;
}

export function afterTransition(
  element: HTMLElement,
  timeoutMs = SETTLE_FALLBACK_MS
): Promise<void> {
  const style = getComputedStyle(element);
  if (transitionMs(style.transitionDuration, style.transitionDelay) <= 0) return Promise.resolve();
  return new Promise((resolve) => {
    const listening = new AbortController();

    const finish = (): void => {
      clearTimeout(timer);
      listening.abort();
      resolve();
    };
    const timer = setTimeout(finish, timeoutMs);
    element.addEventListener(
      'transitionend',
      (event) => {
        if (event.target === element) finish();
      },
      { signal: listening.signal }
    );
  });
}
