<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { Button } from '@immich/ui';
  import RangeSlider from './RangeSlider.svelte';

  let {
    label,
    value = $bindable(),
    min,
    max,
    step = 0.1,
    onLive,
    onCommit,
    onPreviewStart,
    onPreviewEnd,
    format = (v: number): string => v.toFixed(2),
    gradient,
    defaultValue = 0,
    disabled = false,
    commitAction
  }: {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    onLive: (value: number) => void;
    onCommit: (action?: string) => void;
    onPreviewStart?: () => void;
    onPreviewEnd?: () => void;
    format?: (v: number) => string;
    gradient?: string;
    defaultValue?: number;
    disabled?: boolean;
    commitAction?: string;
  } = $props();

  const isDefault = $derived(value === defaultValue);
  const supportsPreview = $derived(!!onPreviewStart && !!onPreviewEnd);
  const displayValue = $derived(format(Object.is(value, -0) ? 0 : value));
  let dragging = $state(false);
  let altDown = $state(false);
  let previewing = $state(false);
  let editing = $state(false);
  let draft = $state(0);
  let editStart = 0;
  let valueInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (!editing || !valueInput) return;
    valueInput.focus();
    valueInput.select();
  });

  function normalize(next: number): number {
    const clamped = Math.min(max, Math.max(min, next));
    return Object.is(clamped, -0) ? 0 : clamped;
  }

  function reset(): void {
    if (isDefault) return;
    value = defaultValue;
    onLive(defaultValue);
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
    const wantPreview = dragging && altDown && !disabled;
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
    altDown = e.altKey;
    updatePreview();
    window.addEventListener('pointerup', onPointerUp, { once: true });
    window.addEventListener('keydown', onKeyChange);
    window.addEventListener('keyup', onKeyChange);
  }

  function onPointerUp(): void {
    dragging = false;
    window.removeEventListener('keydown', onKeyChange);
    window.removeEventListener('keyup', onKeyChange);
    if (previewing) {
      previewing = false;
      onPreviewEnd!();
    }
  }

  function onKeyChange(e: KeyboardEvent): void {
    altDown = e.altKey;
    updatePreview();
  }

  function onInput(e: Event): void {
    const v = normalize((e.currentTarget as HTMLInputElement).valueAsNumber);
    value = v;
    if (previewing) {
      onPreviewStart!();
    } else {
      onLive(v);
    }
  }
</script>

<div
  class="panel-row group h-7 items-center rounded-sm px-0.5 transition-colors hover:bg-white/3 {disabled
    ? 'pointer-events-none opacity-50'
    : ''}"
>
  <Button
    variant="ghost"
    color="secondary"
    size="tiny"
    class="h-6 min-w-0 justify-start gap-1.5 overflow-hidden p-0 text-[11px] font-medium text-dark/65 select-none hover:bg-transparent hover:text-dark"
    title="double click to reset"
    ondblclick={reset}
  >
    <span class="truncate">{label}</span>
  </Button>
  <RangeSlider
    {label}
    {min}
    {max}
    {step}
    {disabled}
    {value}
    {gradient}
    onpointerdown={onPointerDown}
    oninput={onInput}
    onchange={() => onCommit(commitAction)}
    ondblclick={reset}
  />
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
      class="h-5 min-h-0 w-full justify-end rounded px-1 py-0 text-right font-mono text-[10px] tabular-nums {isDefault
        ? 'text-dark/65'
        : 'bg-primary/10 text-primary'}"
      aria-label="Edit {label} value"
      onclick={beginEdit}
    >
      {displayValue}
    </Button>
  {/if}
</div>
