import { goto, type AfterNavigate, type NavigationType } from '$app/navigation';

type NavigationStep = {
  type: Exclude<NavigationType, 'leave'>;
  delta?: number;
  pushed: boolean;
};

type NavigationEntry = { index: number; url: string | null };

type NavigationSource = {
  navigation?: {
    currentEntry?: NavigationEntry | null;
    entries?: () => NavigationEntry[];
  };
};

type HistorySource = { history?: { length?: number } };

type Located = { url: URL | null } | null;

export function nextDepth(depth: number, step: NavigationStep): number {
  switch (step.type) {
    case 'enter':
      return 0;
    case 'popstate':
      return Math.max(0, depth + (step.delta ?? 0));
    default:
      return step.pushed ? depth + 1 : depth;
  }
}

export function hasInAppPrevious(entryIndex: number | null, depth: number): boolean {
  return entryIndex === null ? depth > 0 : entryIndex > 0;
}

export function routeKey(url: URL): string {
  return `${url.pathname}${url.search}`;
}

function targetKey(target: string): string {
  return routeKey(new URL(target, 'http://app.invalid'));
}

function keyOf(target: Located): string | null {
  return target?.url ? routeKey(target.url) : null;
}

function navigationApi(): NavigationSource['navigation'] | null {
  return (globalThis as NavigationSource).navigation ?? null;
}

function currentEntry(): NavigationEntry | null {
  const entry = navigationApi()?.currentEntry;
  return entry && entry.index >= 0 ? entry : null;
}

function historyLength(): number {
  return (globalThis as HistorySource).history?.length ?? 0;
}

function apiPreviousKey(): string | null | undefined {
  const api = navigationApi();
  const current = currentEntry();
  if (!api?.entries || !current) return undefined;
  const url = api.entries()[current.index - 1]?.url;
  if (!url || !current.url) return null;
  const previous = new URL(url);
  return previous.origin === new URL(current.url).origin ? routeKey(previous) : null;
}

class NavHistory {
  #depth = 0;
  #length = 0;
  #previous: string | null = null;
  #arrivals: (() => void)[] = [];

  record = (navigation: AfterNavigate): void => {
    const length = historyLength();
    const pushed = length > this.#length;
    this.#length = length;
    if (navigation.type === 'enter' || navigation.type === 'popstate') this.#previous = null;
    else if (pushed) this.#previous = keyOf(navigation.from);
    this.#depth = nextDepth(this.#depth, {
      type: navigation.type,
      delta: navigation.type === 'popstate' ? navigation.delta : undefined,
      pushed
    });
    const arrivals = this.#arrivals;
    this.#arrivals = [];
    for (const arrive of arrivals) arrive();
  };

  canPop = (): boolean => hasInAppPrevious(currentEntry()?.index ?? null, this.#depth);

  previousIs = (target: string): boolean => {
    const previous = apiPreviousKey();
    const key = previous === undefined ? (this.canPop() ? this.#previous : null) : previous;
    return key !== null && key === targetKey(target);
  };

  back = (parent: string): void => {
    if (this.canPop()) {
      history.back();
      return;
    }
    void goto(parent, { replace: true });
  };

  returnTo = (target: string): Promise<void> => {
    if (!this.previousIs(target)) return goto(target, { replace: true });
    const arrived = new Promise<void>((resolve) => this.#arrivals.push(resolve));
    history.back();
    return arrived;
  };
}

export const navHistory = new NavHistory();
