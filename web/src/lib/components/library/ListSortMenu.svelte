<script lang="ts" generics="T extends string">
  import Popover from '#lib/components/Popover.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiSort } from '@mdi/js';

  let {
    noun,
    options,
    value,
    onChange
  }: {
    noun: string;
    options: readonly { value: T; label: string }[];
    value: T;
    onChange: (sort: T) => void;
  } = $props();

  let open = $state(false);

  const title = $derived(`Sort ${noun}`);
  const label = $derived(
    `${title}: ${options.find((option) => option.value === value)?.label ?? ''}`
  );
</script>

<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  appearance="panel"
  {title}
  contentClass="flex flex-col p-3"
  sheetClass="flex flex-col px-4 py-3"
  tooltip={label}
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="touch:size-11"
      icon={mdiSort}
      title=""
      aria-label={label}
      {...popoverProps}
    />
  {/snippet}
  <SegmentedChoice {options} {value} label={title} {onChange} />
</Popover>
