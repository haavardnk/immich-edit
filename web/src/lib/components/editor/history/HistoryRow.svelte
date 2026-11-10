<script lang="ts">
  import type { EditHistoryEntry } from '#lib/api/edits.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { stepText, type StepState } from '#lib/editor/historySteps.ts';
  import { formatWhen } from '#lib/utils/datetime.ts';
  import { historyDetails, historyLabel } from '#lib/utils/history.ts';
  import { historyIcon } from '#lib/utils/historyIcon.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight, mdiRedo } from '@mdi/js';
  import HistoryDetails from './HistoryDetails.svelte';

  let {
    entry,
    previous,
    isRaw,
    step,
    busy,
    restoring,
    expanded,
    onrestore,
    onhover,
    onleave,
    ontoggle
  }: {
    entry: EditHistoryEntry;
    previous: EditHistoryEntry | null;
    isRaw: boolean;
    step: StepState;
    busy: boolean;
    restoring: boolean;
    expanded: boolean;
    onrestore: () => void;
    onhover: () => void;
    onleave: () => void;
    ontoggle: () => void;
  } = $props();

  const info = $derived(historyLabel(entry, previous, isRaw));
  const details = $derived(historyDetails(entry, previous, isRaw));
  const icon = $derived(historyIcon(entry, details));
  const text = $derived(stepText(entry, previous, isRaw));
  const faded = $derived(step.undone || step.superseded);
  const tag = $derived(
    restoring ? 'restoring…' : step.previewing ? 'previewing' : step.current ? 'current' : null
  );
  const detailsId = $derived(`history-details-${entry.id}`);

  function enter(event: PointerEvent): void {
    if (event.pointerType === 'mouse') onhover();
  }

  function focus(event: FocusEvent): void {
    const row = event.currentTarget;
    if (layout.mode !== 'touch' && row instanceof HTMLElement && row.matches(':focus-visible')) {
      onhover();
    }
  }
</script>

<li
  class="border-b border-dark/5 last:border-b-0 touch:border-hairline"
  data-history-step={entry.id}
  data-undone={step.undone || undefined}
  data-superseded={step.superseded || undefined}
>
  <div
    class="relative flex {step.previewing
      ? 'bg-light-200'
      : step.current
        ? 'bg-light-100 touch:bg-primary/10'
        : ''}"
  >
    {#if step.current}
      <span
        class="absolute inset-y-0 left-0 hidden w-[3px] bg-primary touch:block"
        aria-hidden="true"
      ></span>
    {/if}
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      class="min-w-0 flex-1 justify-start gap-3 rounded-none py-2 pr-1 pl-3 text-left font-normal touch:h-auto touch:py-2.5 touch:pr-4 touch:pl-4"
      aria-label={step.undone ? `Jump forward to ${text}` : `Jump to ${text}`}
      aria-current={step.current ? 'step' : undefined}
      disabled={busy}
      onpointerenter={enter}
      onfocus={focus}
      onblur={onleave}
      onclick={onrestore}
    >
      <span
        class="hidden size-8 shrink-0 items-center justify-center rounded-lg touch:flex {step.current
          ? 'bg-primary/20 text-primary'
          : 'bg-light-200 text-dark/80'} {faded ? 'opacity-50' : ''}"
        aria-hidden="true"
      >
        <Icon {icon} size="18px" />
      </span>
      <span class="flex min-w-0 flex-1 flex-col items-start gap-0.5">
        <span class="flex w-full min-w-0 items-center gap-1.5 text-xs text-dark touch:text-base">
          <span
            class="flex min-w-0 items-center gap-1.5 {faded
              ? 'line-through opacity-50 touch:no-underline'
              : ''}"
          >
            <span class="truncate touch:font-medium">{info.label}</span>
            {#if info.delta}
              <span
                class="shrink-0 font-mono text-dark/65 tabular-nums touch:hidden"
                aria-hidden="true">{info.delta}</span
              >
            {/if}
          </span>
          {#if info.delta}
            <span
              class="ml-auto hidden shrink-0 text-dark tabular-nums touch:inline {faded
                ? 'opacity-50'
                : ''}"
              aria-hidden="true">{info.delta}</span
            >
          {/if}
          <span
            class="ml-auto flex shrink-0 items-center justify-end gap-1.5 touch:w-20 {info.delta
              ? 'touch:ml-0'
              : ''}"
          >
            {#if tag}
              <span
                class="text-[9px] tracking-wider uppercase touch:text-[11px] touch:font-semibold {tag ===
                'previewing'
                  ? 'font-semibold text-primary'
                  : step.current
                    ? 'text-dark/65 touch:text-primary'
                    : 'text-dark/65'}">{tag}</span
              >
            {/if}
            {#if step.undone}
              <Icon icon={mdiRedo} size="14px" class="shrink-0 text-dark/45" aria-hidden="true" />
            {/if}
          </span>
        </span>
        <span class="text-[10px] text-dark/65 touch:hidden {faded ? 'opacity-50' : ''}"
          >{formatWhen(entry.created_at)}</span
        >
      </span>
    </Button>
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="w-9 shrink-0 rounded-none touch:hidden"
      icon={expanded ? mdiChevronDown : mdiChevronRight}
      disabled={busy || details.length === 0}
      onclick={ontoggle}
      aria-label={`${expanded ? 'Collapse' : 'Expand'} changes for ${info.label}`}
      aria-expanded={expanded}
      aria-controls={detailsId}
    />
  </div>
  {#if expanded}
    <HistoryDetails id={detailsId} {details} />
  {/if}
</li>
