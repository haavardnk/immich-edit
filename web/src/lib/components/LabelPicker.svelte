<script lang="ts">
  import LabelSwatches from '#lib/components/LabelSwatches.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { LABEL_NAMES, type LabelColor } from '#lib/stores/labels.ts';
  import { labelLook } from '#lib/browse/bulk/bulkMarks.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';

  let {
    label,
    onchange,
    disabled = false,
    size = 'tiny'
  }: {
    label: LabelColor | null;
    onchange: (color: LabelColor | null) => void;
    disabled?: boolean;
    size?: 'tiny' | 'small' | 'medium';
  } = $props();

  let open = $state(false);

  function pick(color: LabelColor | null): void {
    open = false;
    onchange(color);
  }
</script>

<Popover
  {open}
  onOpenChange={(v) => (open = v)}
  appearance="control"
  anchor="top"
  title="Colour label"
  tooltip={hint(label ? `${LABEL_NAMES[label]} label` : 'Colour label', 'label')}
>
  {#snippet trigger(popoverProps)}
    <IconButton
      {...popoverProps}
      {size}
      data-label={label ?? undefined}
      variant="ghost"
      color="secondary"
      class={labelLook(label).tone}
      icon={labelLook(label).icon}
      title=""
      aria-label={label ? `${LABEL_NAMES[label]} label` : 'Colour label'}
      aria-expanded={open}
      {disabled}
    />
  {/snippet}
  <LabelSwatches {label} onpick={pick} />
</Popover>
