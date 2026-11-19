import type { CullState } from '#lib/browse/cullState.ts';
import { LoupeStage, type LoupeStageSpec, type StageSurface } from '#lib/browse/loupeStage.ts';
import type { StepDelta } from '#lib/browse/loupeSwipe.ts';
import { ADVANCE_MS, ratingHud } from '#lib/browse/ratingHud.svelte.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

type Surface = StageSurface & {
  style: { transform: string; transition: string; opacity: string };
};

function surface(): Surface {
  return {
    clientWidth: 400,
    clientHeight: 800,
    style: { transform: '', transition: '', opacity: '' }
  };
}

function setup(overrides: Partial<LoupeStageSpec> = {}) {
  const track = surface();
  const scrim = surface();
  const spec = {
    track: () => track,
    scrim: () => scrim,
    canStep: (_delta: StepDelta) => true,
    hasNeighbour: (_delta: StepDelta) => true,
    current: (): string | null => 'a',
    step: vi.fn(),
    dismiss: vi.fn(),
    openInfo: vi.fn(),
    reducedMotion: () => false,
    ...overrides
  };
  return { track, scrim, spec, stage: new LoupeStage(spec) };
}

function strip(ids: string[]) {
  let shown = ids[0] ?? null;
  return {
    current: (): string | null => shown,
    show: (id: string): void => {
      shown = id;
    },
    step: vi.fn((delta: StepDelta): void => {
      shown = ids[ids.indexOf(shown ?? '') + delta] ?? shown;
    })
  };
}

const RATED: CullState = {
  rating: 3,
  isFavorite: false,
  rejected: false,
  label: null,
  owned: true
};

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  ratingHud.clear();
  vi.useRealTimers();
});

describe('LoupeStage', () => {
  it('slides the photo with the finger and steps once it has slid out', () => {
    const { track, spec, stage } = setup();
    stage.begin({ dx: -40, dy: 0, axis: 'x' });
    stage.follow({ dx: -150, dy: 4, axis: 'x' });
    expect(track.style.transform).toBe('translate3d(-150px, 0, 0)');

    stage.release({ direction: 'left', dx: -150, dy: 4, velocity: -0.2 });
    expect(track.style.transform).toBe('translate3d(-416px, 0, 0)');
    expect(track.style.transition).toContain('transform');
    expect(spec.step).not.toHaveBeenCalled();

    vi.runAllTimers();
    expect(spec.step).toHaveBeenCalledExactlyOnceWith(1);
    expect(track.style.transform).toBe('');
    expect(track.style.transition).toBe('');
  });

  it('springs back from a short swipe without stepping', () => {
    const { track, spec, stage } = setup();
    stage.begin({ dx: 40, dy: 0, axis: 'x' });
    stage.release({ direction: null, dx: 40, dy: 0, velocity: 0.1 });
    vi.runAllTimers();
    expect(spec.step).not.toHaveBeenCalled();
    expect(track.style.transform).toBe('');
  });

  it('rubber-bands and springs back at the first photo', () => {
    const { track, spec, stage } = setup({ canStep: (delta) => delta === 1 });
    stage.begin({ dx: 300, dy: 0, axis: 'x' });
    const pulled = Number(/-?[\d.]+/.exec(track.style.transform)?.[0]);
    expect(pulled).toBeGreaterThan(0);
    expect(pulled).toBeLessThan(300);
    stage.release({ direction: 'right', dx: 300, dy: 0, velocity: 2 });
    vi.runAllTimers();
    expect(spec.step).not.toHaveBeenCalled();
  });

  it('springs back and asks for more when the neighbour is not loaded yet', () => {
    const { spec, stage } = setup({ hasNeighbour: () => false });
    stage.begin({ dx: -200, dy: 0, axis: 'x' });
    stage.release({ direction: 'left', dx: -200, dy: 0, velocity: -1 });
    expect(spec.step).toHaveBeenCalledExactlyOnceWith(1);
  });

  it('shrinks the photo and fades the scrim while pulled down, then dismisses', () => {
    const { track, scrim, spec, stage } = setup();
    stage.begin({ dx: 0, dy: 200, axis: 'y' });
    expect(track.style.transform).toBe('translate3d(0, 200px, 0) scale(0.925)');
    expect(scrim.style.opacity).toBe('0.5');
    stage.release({ direction: 'down', dx: 0, dy: 200, velocity: 0.1 });
    expect(spec.dismiss).toHaveBeenCalledOnce();
    expect(track.style.transform).toBe('');
    expect(scrim.style.opacity).toBe('');
  });

  it('opens the info on a swipe up and settles the photo', () => {
    const { track, spec, stage } = setup();
    stage.begin({ dx: 0, dy: -80, axis: 'y' });
    stage.release({ direction: 'up', dx: 0, dy: -80, velocity: -0.6 });
    expect(spec.openInfo).toHaveBeenCalledOnce();
    expect(spec.dismiss).not.toHaveBeenCalled();
    vi.runAllTimers();
    expect(track.style.transform).toBe('');
  });

  it('steps at once without a slide when motion is reduced', () => {
    const { track, spec, stage } = setup({ reducedMotion: () => true });
    stage.begin({ dx: -200, dy: 0, axis: 'x' });
    stage.release({ direction: 'left', dx: -200, dy: 0, velocity: -1 });
    expect(spec.step).toHaveBeenCalledExactlyOnceWith(1);
    expect(track.style.transform).toBe('');
    expect(track.style.transition).toBe('');
  });

  it('finishes a running slide before the next swipe begins', () => {
    const { spec, stage } = setup();
    stage.begin({ dx: -200, dy: 0, axis: 'x' });
    stage.release({ direction: 'left', dx: -200, dy: 0, velocity: -1 });
    stage.begin({ dx: -20, dy: 0, axis: 'x' });
    expect(spec.step).toHaveBeenCalledExactlyOnceWith(1);
    vi.runAllTimers();
    expect(spec.step).toHaveBeenCalledOnce();
  });

  it('never steps after it is disposed', () => {
    const { spec, stage } = setup();
    stage.begin({ dx: -200, dy: 0, axis: 'x' });
    stage.release({ direction: 'left', dx: -200, dy: 0, velocity: -1 });
    stage.dispose();
    vi.runAllTimers();
    expect(spec.step).not.toHaveBeenCalled();
  });

  it('springs back when the gesture is cancelled', () => {
    const { track, stage } = setup();
    stage.begin({ dx: -200, dy: 0, axis: 'x' });
    stage.cancel();
    vi.runAllTimers();
    expect(track.style.transform).toBe('');
  });

  it('steps exactly once when a rating countdown would run out during the slide', () => {
    const photos = strip(['a', 'b', 'c']);
    const { stage } = setup({ current: photos.current, step: photos.step });
    ratingHud.show('a', 'rating', RATED, () => photos.step(1));
    vi.advanceTimersByTime(ADVANCE_MS - 50);

    stage.begin({ dx: -200, dy: 0, axis: 'x' });
    expect(ratingHud.shown).toBeNull();
    stage.release({ direction: 'left', dx: -200, dy: 0, velocity: -1 });
    vi.runAllTimers();

    expect(photos.step).toHaveBeenCalledExactlyOnceWith(1);
    expect(photos.current()).toBe('b');
  });

  it('only settles when something else stepped during the slide', () => {
    const photos = strip(['a', 'b', 'c']);
    const { track, stage } = setup({ current: photos.current, step: photos.step });
    stage.begin({ dx: -200, dy: 0, axis: 'x' });
    stage.release({ direction: 'left', dx: -200, dy: 0, velocity: -1 });
    photos.show('b');
    vi.runAllTimers();

    expect(photos.step).not.toHaveBeenCalled();
    expect(photos.current()).toBe('b');
    expect(track.style.transform).toBe('');
  });
});
