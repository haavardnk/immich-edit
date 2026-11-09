<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button } from '@immich/ui';
  import RangeSlider from './RangeSlider.svelte';
  import { SCALE_POSITIONS } from './sliderScale';
  import type { SliderRange } from '#lib/editor/params/types.ts';
  import { keyHeld, latched } from '#lib/input/modifiers.svelte.ts';
  import { untrack } from 'svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { panRecognizer } from '#lib/gestures/pan.ts';
  import { tapRecognizer } from '#lib/gestures/tap.ts';
  import { dragBounds, dragPosition, valueAt } from './relativeDrag';

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
    commitAction
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
  } = $props();

  const { min, max, step, scale } = $derived(range);
  const resetValue = $derived(defaultValue ?? range.defaultValue ?? 0);
  const isDefault = $derived(value === resetValue);
  const supportsPreview = $derived(!!onPreviewStart && !!onPreviewEnd);
  const displayValue = $derived((format ?? range.format)(Object.is(value, -0) ? 0 : value));
  const position = $derived(scale ? scale.toPosition(value) : value);
  let dragging = $state(false);
  let previewKey = $state(false);
  const previewHeld = $derived(previewKey || latched('preview'));
  let previewing = $state(false);
  let editing = $state(false);
  let draft = $state(0);
  let editStart = 0;
  let valueInput = $state<HTMLInputElement | null>(null);
  let track = $state<HTMLDivElement | null>(null);
  const touch = $derived(layout.mode === 'touch');
  let relative: { position: number; centreY: number; start: number } | null = null;

  $effect(() => {
    if (!editing || !valueInput) return;
    valueInput.focus();
    valueInput.select();
  });

  $effect(() => {
    void previewHeld;
    untrack(updatePreview);
  });

  function normalize(next: number): number {
    const clamped = Math.min(max, Math.max(min, next));
    return Object.is(clamped, -0) ? 0 : clamped;
  }

  function reset(): void {
    if (isDefault) return;
    value = resetValue;
    onLive(resetValue);
    onCommit(commitAction);
  }

  function beginEdit(): void {
    if (disabled) return;
    editStart = value;
    draft = value;
    editing = true;
  }

  function onDraftInput(e: Event): void {
    const next = (e.currentTarget as HTMLInputElement).valueAsNumber;
    draft = next;
    if (!Number.isFinite(next)) return;
    value = normalize(next);
    onLive(value);
  }

  function commitEdit(): void {
    if (!editing) return;
    const next = Number.isFinite(draft) ? normalize(draft) : editStart;
    value = next;
    onLive(next);
    editing = false;
    if (next !== editStart) onCommit(commitAction);
  }

  function cancelEdit(): void {
    value = editStart;
    draft = editStart;
    onLive(editStart);
    editing = false;
  }

  function updatePreview(): void {
    if (!supportsPreview) return;
    const wantPreview = dragging && previewHeld && !disabled;
    if (wantPreview && !previewing) {
      previewing = true;
      onPreviewStart!();
    } else if (!wantPreview && previewing) {
      previewing = false;
      onPreviewEnd!();
    }
  }

  function onPointerDown(e: PointerEvent): void {
    if (disabled) return;
    dragging = true;
    previewKey = keyHeld('preview', e);
    editor.beginDrag();
    onDragStart?.();
    updatePreview();
    window.addEventListener('pointerup', onPointerUp);
    window.addEventListener('pointercancel', onPointerUp);
    window.addEventListener('keydown', onKeyChange);
    window.addEventListener('keyup', onKeyChange);
  }

  function onPointerUp(): void {
    dragging = false;
    window.removeEventListener('pointerup', onPointerUp);
    window.removeEventListener('pointercancel', onPointerUp);
    window.removeEventListener('keydown', onKeyChange);
    window.removeEventListener('keyup', onKeyChange);
    editor.endDrag();
    onDragEnd?.();
    if (previewing) {
      previewing = false;
      onPreviewEnd!();
    }
  }

  function relativeStart(): void {
    if (!track) return;
    const box = track.getBoundingClientRect();
    relative = { position, centreY: box.top + box.height / 2, start: value };
    editor.beginDrag();
    onDragStart?.();
  }

  function relativeMove(stepX: number, y: number): void {
    if (!relative || !track) return;
    relative.position = dragPosition(relative.position, stepX, y - relative.centreY, {
      ...dragBounds(range),
      width: track.clientWidth
    });
    const next = valueAt(relative.position, range);
    if (next === value) return;
    value = next;
    onLive(next);
  }

  function relativeEnd(): void {
    const current = relative;
    if (!current) return;
    relative = null;
    editor.endDrag();
    onDragEnd?.();
    if (value !== current.start) onCommit(commitAction);
  }

  const touchDrag = gestures({
    pointerTypes: ['touch', 'pen'],
    touchAction: 'pan-y',
    recognizers: [
      panRecognizer({
        enabled: () => touch && !disabled,
        lockAxis: true,
        thresholds: { panSlop: 8 },
        start: (pan) => {
          if (pan.axis === 'x') relativeStart();
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

  function onKeyChange(e: KeyboardEvent): void {
    previewKey = keyHeld('preview', e);
    updatePreview();
  }

  function onInput(e: Event): void {
    const raw = (e.currentTarget as HTMLInputElement).valueAsNumber;
    const v = normalize(scale ? scale.fromPosition(raw) : raw);
    value = v;
    if (previewing) {
      onPreviewStart!();
    } else {
      onLive(v);
    }
  }
</script>

<div
  class="panel-row group h-7 items-center rounded-sm px-0.5 transition-colors hover:bg-light-950/3 touch:h-11 touch:grid-cols-[6rem_minmax(0,1fr)_3.25rem] {disabled
    ? 'pointer-events-none opacity-50'
    : ''}"
>
  <Button
    variant="ghost"
    color="secondary"
    size="tiny"
    class="h-6 min-w-0 justify-start gap-1.5 overflow-hidden p-0 text-[11px] font-medium text-dark/65 select-none hover:bg-transparent hover:text-dark touch:h-11 touch:text-[13px]"
    title={touch ? '' : 'double click to reset'}
    aria-label={touch && onLabelTap ? `Focus ${label}` : undefined}
    onclick={onLabelClick}
    ondblclick={reset}
  >
    <span class="truncate">{label}</span>
  </Button>
  <div bind:this={track} class="flex min-w-0 items-center touch:h-11" {@attach touchDrag}>
    <RangeSlider
      class="w-full touch:pointer-events-none"
      {label}
      min={scale ? 0 : min}
      max={scale ? SCALE_POSITIONS : max}
      step={scale ? 1 : step}
      defaultValue={scale ? scale.toPosition(resetValue) : resetValue}
      {disabled}
      value={position}
      valueText={scale ? displayValue : undefined}
      gradient={gradient ?? range.gradient}
      title={supportsPreview ? `${keysFor('sliderPreviewDrag')} to preview` : undefined}
      onpointerdown={onPointerDown}
      oninput={onInput}
      onchange={() => onCommit(commitAction)}
      ondblclick={reset}
    />
  </div>
  {#if editing}
    <TextInput
      bind:ref={valueInput}
      type="number"
      size="tiny"
      class="h-5 bg-hairline [&_input]:px-1 [&_input]:py-0 [&_input]:text-right [&_input]:font-mono [&_input]:text-[10px] [&_input]:tabular-nums"
      aria-label="{label} value"
      value={String(draft)}
      {min}
      {max}
      {step}
      oninput={onDraftInput}
      onblur={commitEdit}
      onkeydown={(e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          commitEdit();
        } else if (e.key === 'Escape') {
          e.preventDefault();
          cancelEdit();
        }
      }}
    />
  {:else}
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      class="h-5 min-h-0 w-full justify-end rounded px-1 py-0 text-right font-mono text-[10px] tabular-nums touch:h-8 touch:text-xs {isDefault
        ? 'text-dark/65'
        : 'bg-primary/10 text-primary'}"
      aria-label="Edit {label} value"
      onclick={beginEdit}
    >
      {displayValue}
    </Button>
  {/if}
</div>
