<script lang="ts">
  import {
    boundAt,
    boundDay,
    boundTime,
    joinBound,
    type DateBoundKey
  } from '#lib/browse/dateBounds.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { Field, IconButton, TimeInput, getLocale } from '@immich/ui';
  import { Time } from '@internationalized/date';
  import { mdiClose } from '@mdi/js';
  import type { TimeValue } from 'bits-ui';
  import { tick } from 'svelte';

  let { key, label, onCleared }: { key: DateBoundKey; label: string; onCleared: () => void } =
    $props();

  let addingTime = $state(false);
  let row: HTMLDivElement | undefined = $state();

  const bound = $derived(browseControls.filters[key]);
  const day = $derived(boundDay(bound));
  const time = $derived(boundTime(bound));
  const dayText = $derived(
    day
      ? boundAt(day).toLocaleString(
          { day: 'numeric', month: 'short', year: 'numeric' },
          { locale: getLocale() }
        )
      : 'Any'
  );

  function readTime(value: string): Time | undefined {
    const [hour, minute] = value.split(':').map(Number);
    return value ? new Time(hour, minute) : undefined;
  }

  function writeTime(value: TimeValue | undefined): void {
    const next = value
      ? `${String(value.hour).padStart(2, '0')}:${String(value.minute).padStart(2, '0')}`
      : '';
    browseControls.filters[key] = joinBound(day, next);
  }

  async function addTime(): Promise<void> {
    addingTime = true;
    await tick();
    row?.querySelector<HTMLElement>('[data-segment="hour"]')?.focus();
  }

  function clear(): void {
    addingTime = false;
    browseControls.filters[key] = '';
    onCleared();
  }
</script>

<div
  bind:this={row}
  role="group"
  aria-label={label}
  class="flex h-8 items-center gap-2 text-sm touch:h-11"
>
  <span class="w-9 shrink-0 text-muted">{label}</span>
  <span class={['min-w-0 flex-1 truncate', !day && 'text-muted']}>{dayText}</span>
  {#if day && (time || addingTime)}
    <div role="group" aria-label="{label} time" class="compact-date-field shrink-0">
      <Field size="tiny">
        <TimeInput granularity="minute" bind:value={() => readTime(time), writeTime} />
      </Field>
    </div>
  {:else if day}
    <button
      type="button"
      class="shrink-0 rounded-full px-2 py-0.5 text-xs text-primary hover:bg-primary/10"
      onclick={addTime}
    >
      Add time
    </button>
  {/if}
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    shape="round"
    icon={mdiClose}
    title="Clear {label.toLowerCase()}"
    aria-label="Clear {label.toLowerCase()}"
    disabled={!day}
    class={day ? '' : 'invisible'}
    onclick={clear}
  />
</div>
