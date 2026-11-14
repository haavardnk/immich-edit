import {
  dismisses,
  dismissFrame,
  exitOffset,
  followX,
  swipeStep,
  type StepDelta
} from '#lib/browse/loupeSwipe.ts';
import { ratingHud } from '#lib/browse/ratingHud.svelte.ts';
import type { Axis } from '#lib/gestures/axis.ts';
import type { SwipeEnd, SwipeProgress } from '#lib/gestures/swipe.ts';

export type StageSurface = {
  readonly clientWidth: number;
  readonly clientHeight: number;
  readonly style: Pick<CSSStyleDeclaration, 'transform' | 'transition' | 'opacity'>;
};

export type LoupeStageSpec = {
  track: () => StageSurface | null;
  scrim: () => StageSurface | null;
  canStep: (delta: StepDelta) => boolean;
  hasNeighbour: (delta: StepDelta) => boolean;
  current: () => string | null;
  step: (delta: StepDelta) => void;
  dismiss: () => void;
  openInfo: () => void;
  reducedMotion: () => boolean;
};

type Settle = { timer: ReturnType<typeof setTimeout>; done: () => void };

const SETTLE_MS = 200;

const EASE = `${SETTLE_MS}ms cubic-bezier(0.2, 0.8, 0.2, 1)`;

export class LoupeStage {
  #axis: Axis | null = null;
  #from: string | null = null;
  #settle: Settle | null = null;

  constructor(private readonly spec: LoupeStageSpec) {}

  begin(progress: SwipeProgress): void {
    this.#finish();
    ratingHud.clear();
    this.#from = this.spec.current();
    this.#axis = progress.axis;
    this.follow(progress);
  }

  follow(progress: SwipeProgress): void {
    const track = this.spec.track();
    if (!track || !this.#axis) return;
    if (this.#axis === 'x') {
      const x = followX(progress.dx, track.clientWidth, this.spec.canStep);
      this.#paint(`translate3d(${x}px, 0, 0)`, '');
      return;
    }
    const frame = dismissFrame(progress.dy, track.clientHeight);
    this.#paint(`translate3d(0, ${frame.offset}px, 0) scale(${frame.scale})`, String(frame.fade));
  }

  release(end: SwipeEnd): void {
    const axis = this.#axis;
    this.#axis = null;
    const track = this.spec.track();
    if (!track || !axis) return;
    if (axis === 'x') {
      this.#settleSwipe(end, track.clientWidth);
      return;
    }
    if (dismisses(end.dy, end.velocity, track.clientHeight)) {
      this.spec.dismiss();
      this.#rest();
      return;
    }
    if (end.direction === 'up') this.spec.openInfo();
    this.#springBack();
  }

  cancel(): void {
    if (!this.#axis) return;
    this.#axis = null;
    this.#springBack();
  }

  dispose(): void {
    if (this.#settle) clearTimeout(this.#settle.timer);
    this.#settle = null;
    this.#axis = null;
  }

  #settleSwipe(end: SwipeEnd, width: number): void {
    const delta = swipeStep(end.dx, end.velocity, width, this.spec.canStep);
    if (delta === null) {
      this.#springBack();
      return;
    }
    if (!this.spec.hasNeighbour(delta)) {
      this.#springBack();
      this.spec.step(delta);
      return;
    }
    const from = this.#from;
    this.#animate(`translate3d(${exitOffset(delta, width)}px, 0, 0)`, () => {
      if (this.spec.current() === from) this.spec.step(delta);
      this.#rest();
    });
  }

  #springBack(): void {
    this.#animate('', () => this.#rest());
  }

  #animate(transform: string, done: () => void): void {
    if (this.spec.reducedMotion()) {
      done();
      return;
    }
    this.#transition(`transform ${EASE}`, `opacity ${EASE}`);
    this.#paint(transform, '');
    this.#settle = {
      timer: setTimeout(() => {
        this.#settle = null;
        done();
      }, SETTLE_MS),
      done
    };
  }

  #finish(): void {
    const settle = this.#settle;
    if (!settle) return;
    clearTimeout(settle.timer);
    this.#settle = null;
    settle.done();
  }

  #rest(): void {
    this.#transition('', '');
    this.#paint('', '');
  }

  #transition(track: string, scrim: string): void {
    const trackSurface = this.spec.track();
    const scrimSurface = this.spec.scrim();
    if (trackSurface) trackSurface.style.transition = track;
    if (scrimSurface) scrimSurface.style.transition = scrim;
  }

  #paint(transform: string, fade: string): void {
    const track = this.spec.track();
    const scrim = this.spec.scrim();
    if (track) track.style.transform = transform;
    if (scrim) scrim.style.opacity = fade;
  }
}
