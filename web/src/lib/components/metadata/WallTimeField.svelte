<script lang="ts">
  import { DatePicker, Field, TimeInput } from '@immich/ui';
  import { Time } from '@internationalized/date';
  import type { TimeValue } from 'bits-ui';
  import { DateTime } from 'luxon';

  let {
    label,
    value,
    onChange
  }: {
    label: string;
    value: number;
    onChange: (wall: number) => void;
  } = $props();

  const at = $derived(DateTime.fromMillis(value, { zone: 'utc' }));

  function writeDay(day: DateTime | undefined): void {
    if (!day) return;
    onChange(at.set({ year: day.year, month: day.month, day: day.day }).toMillis());
  }

  function writeTime(time: TimeValue | undefined): void {
    if (!time) return;
    onChange(at.set({ hour: time.hour, minute: time.minute, second: time.second }).toMillis());
  }
</script>

<div role="group" aria-label={label} class="flex min-w-0 items-center gap-2">
  <div class="compact-date-field min-w-0 flex-1">
    <Field size="tiny">
      <DatePicker
        bind:value={
          () => DateTime.fromObject({ year: at.year, month: at.month, day: at.day }), writeDay
        }
      />
    </Field>
  </div>
  <div class="compact-date-field shrink-0">
    <Field size="tiny">
      <TimeInput
        granularity="second"
        bind:value={() => new Time(at.hour, at.minute, at.second), writeTime}
      />
    </Field>
  </div>
</div>
