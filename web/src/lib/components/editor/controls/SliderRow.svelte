<script lang="ts">
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { Button } from '@immich/ui';
  import RangeSlider from './RangeSlider.svelte';
  import SliderValue from './SliderValue.svelte';
  import { SCALE_POSITIONS } from './sliderScale';
  import type { SliderRange } from '#lib/editor/params/types.ts';
  import { untrack, type Snippet } from 'svelte';
  import ContextMenu from '#lib/components/ContextMenu.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { panRecognizer } from '#lib/gestures/pan.ts';
  import { tapRecognizer } from '#lib/gestures/tap.ts';
  import { valueEntry } from '#lib/stores/valueEntry.svelte.ts';
  import { dragBounds, dragPosition, valueAt } from './relativeDrag';
  import { mouseScrub } from './mouseScrub';
  import { SliderSession } from './sliderSession.svelte';
  import { clampTo, nudgeValue } from './sliderStep';
  import { sliderWheel } from './sliderWheel';

  const LABEL_SLOP = 3;

  let {
    label,
    value = $bindable(),
    range,
    defaultValue,
    gradient,
    format,
    onLive,
    onCommit,
    onPreviewStart,
    onPreviewEnd,
    onDragStart,
    onDragEnd,
    onLabelTap,
    disabled = false,
    commitAction,
    setting,
    menu,
    menuTitle
  }: {
    label: string;
    value: number;
    range: SliderRange;
    defaultValue?: number;
    gradient?: string;
    format?: (value: number) => string;
    onLive: (value: number) => void;
    onCommit: (action?: string) => void;
    onPreviewStart?: () => void;
    onPreviewEnd?: () => void;
    onDragStart?: () => void;
    onDragEnd?: () => void;
    onLabelTap?: () => void;
    disabled?: boolean;
    commitAction?: string;
    setting?: string;
    menu?: Snippet;
    menuTitle?: string;
  } = $props();

  const { min, max, step, fineStep, scale } = $derived(range);
  const shown = $derived(format ? { ...range, format } : range);
  const resetValue = $derived(defaultValue ?? range.defaultValue ?? 0);
  const isDefault = $derived(value === resetValue);
  const supportsPreview = $derived(!!onPreviewStart && !!onPreviewEnd);
  const position = $derived(scale ? scale.toPosition(value) : value);
  const touch = $derived(layout.mode === 'touch');
  let row = $state<HTMLDivElement | null>(null);
  let track = $state<HTMLDivElement | null>(null);
  const entryOwner = $derived(valueEntry.target?.owner ?? null);
  const dimmed = $derived(entryOwner !== null && !row?.contains(entryOwner));
  let relative: { position: number; centreY: number; width: number } | null = null;

  const session = new SliderSession({
    value: () => value,
    disabled: () => disabled,
    write: (next) => {
      value = next;
    },
    live: (next) => onLive(next),
    commit: () => onCommit(commitAction),
    previewStart: () => onPreviewStart?.(),
    previewEnd: () => onPreviewEnd?.(),
    dragStart: () => onDragStart?.(),
    dragEnd: () => onDragEnd?.()
  });

  $effect(() => {
    void session.previewHeld;
    if (supportsPreview) untrack(session.sync);
  });

  $effect(() => () => session.end());

  function apply(next: number): void {
    value = next;
    onLive(next);
  }

  function reset(): void {
    if (isDefault) return;
    apply(resetValue);
    onCommit(commitAction);
  }

  function input(): HTMLInputElement | null {
    return track?.querySelector('input') ?? null;
  }

  function thumbOf(element: HTMLInputElement): number {
    return parseFloat(getComputedStyle(element).getPropertyValue('--slider-thumb')) || 0;
  }

  function relativeFrom(at: number): void {
    const element = input();
    if (!element) return;
    const box = element.getBoundingClientRect();
    relative = {
      position: at,
      centreY: box.top + box.height / 2,
      width: box.width - thumbOf(element)
    };
  }

  function relativeMove(stepX: number, y: number): void {
    if (!relative) return;
    relative.position = dragPosition(relative.position, stepX, y - relative.centreY, {
      ...dragBounds(range),
      width: relative.width
    });
    const next = valueAt(relative.position, range);
    if (next !== value) session.set(next);
  }

  function relativeEnd(): void {
    relative = null;
    session.end();
  }

  function pressedAt(e: PointerEvent, element: HTMLInputElement): number {
    const box = element.getBoundingClientRect();
    const thumb = thumbOf(element);
    const travel = box.width - thumb;
    const bounds = dragBounds(range);
    const span = bounds.max - bounds.min;
    if (travel <= 0 || span <= 0) return position;
    const thumbX = box.left + thumb / 2 + ((position - bounds.min) / span) * travel;
    if (Math.abs(e.clientX - thumbX) <= thumb / 2) return position;
    return bounds.min + clampTo((e.clientX - box.left - thumb / 2) / travel, 0, 1) * span;
  }

  function trackStart(e: PointerEvent): boolean {
    const element = input();
    if (!element) return false;
    e.preventDefault();
    element.focus({ preventScroll: true });
    session.begin(e);
    const at = pressedAt(e, element);
    relativeFrom(at);
    const next = valueAt(at, range);
    if (at !== position && next !== value) session.set(next);
    return true;
  }

  function labelStart(e: PointerEvent): boolean {
    session.begin(e);
    relativeFrom(position);
    return true;
  }

  const trackDrag = mouseScrub({
    enabled: () => !disabled,
    slop: 0,
    start: trackStart,
    move: relativeMove,
    end: relativeEnd
  });

  const labelScrub = mouseScrub({
    enabled: () => !disabled,
    slop: LABEL_SLOP,
    start: labelStart,
    move: relativeMove,
    end: relativeEnd
  });

  const wheel = sliderWheel({
    enabled: () => !disabled,
    nudge: (steps, size) => {
      session.begin();
      session.set(nudgeValue(value, range, steps, size));
    },
    settle: session.end
  });

  const touchDrag = gestures({
    pointerTypes: ['touch', 'pen'],
    touchAction: 'pan-y',
    recognizers: [
      panRecognizer({
        enabled: () => touch && !disabled,
        lockAxis: true,
        thresholds: { panSlop: 8 },
        start: (pan) => {
          if (pan.axis !== 'x' || !track) return;
          session.begin();
          const box = track.getBoundingClientRect();
          relative = { position, centreY: box.top + box.height / 2, width: track.clientWidth };
        },
        move: (pan) => relativeMove(pan.stepX, pan.y),
        end: relativeEnd,
        cancel: relativeEnd
      }),
      tapRecognizer({ enabled: () => touch && !disabled, doubleTap: reset })
    ]
  });

  function onLabelClick(): void {
    if (touch && onLabelTap) onLabelTap();
  }

  function onInput(e: Event): void {
    const raw = (e.currentTarget as HTMLInputElement).valueAsNumber;
    session.set(clampTo(scale ? scale.fromPosition(raw) : raw, min, max));
  }
</script>

{#snippet slider(rowProps: Record<string, unknown>, labelProps: Record<string, unknown>)}
  <div
    {...rowProps}
    bind:this={row}
    data-setting={setting}
    class="panel-row group h-7 items-center rounded-sm px-0.5 transition-[background-color,opacity] hover:bg-light-950/3 touch:h-11 touch:grid-cols-[6rem_minmax(0,1fr)_3.25rem] {disabled
      ? 'pointer-events-none opacity-50'
      : dimmed
        ? 'opacity-28'
        : ''}"
    {@attach wheel}
  >
    <Button
      {...labelProps}
      variant="ghost"
      color="secondary"
      size="tiny"
      class="h-6 min-w-0 justify-start gap-1.5 overflow-hidden p-0 text-[11px] font-medium text-dark/65 select-none hover:bg-transparent hover:text-dark touch:h-11 touch:text-[13px] {touch
        ? ''
        : 'cursor-ew-resize'}"
      title={touch ? '' : 'Drag to adjust, double-click to reset'}
      aria-label={touch && onLabelTap ? `Focus ${label}` : undefined}
      onclick={onLabelClick}
      ondblclick={reset}
      {@attach labelScrub}
    >
      <span class="truncate">{label}</span>
    </Button>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      bind:this={track}
      class="flex min-w-0 items-center touch:h-11"
      ondblclick={reset}
      {@attach touchDrag}
      {@attach trackDrag}
    >
      <RangeSlider
        class="w-full touch:pointer-events-none"
        {label}
        min={scale ? 0 : min}
        max={scale ? SCALE_POSITIONS : max}
        step={scale ? 1 : step}
        fineStep={scale ? 1 : fineStep}
        defaultValue={scale ? scale.toPosition(resetValue) : resetValue}
        {disabled}
        value={position}
        valueText={scale ? shown.format(value) : undefined}
        gradient={gradient ?? range.gradient}
        title={supportsPreview ? `${keysFor('sliderPreviewDrag')} to preview` : undefined}
        oninput={onInput}
        onchange={() => onCommit(commitAction)}
      />
    </div>
    <SliderValue
      {label}
      {value}
      range={shown}
      {resetValue}
      modified={!isDefault}
      {disabled}
      onLive={apply}
      onCommit={() => onCommit(commitAction)}
      {setting}
    />
  </div>
{/snippet}

{#if menu}
  <ContextMenu title={menuTitle}>
    {#snippet trigger(props)}
      {@render slider(touch ? {} : props, touch ? props : {})}
    {/snippet}
    {@render menu()}
  </ContextMenu>
{:else}
  {@render slider({}, {})}
{/if}
