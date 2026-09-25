<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import { LABEL_COLORS, LABEL_NAMES, LABEL_TEXT, labelKey, type LabelColor } from '$lib/labels';
  import { hint } from '$lib/keybinds';
  import { IconButton } from '@immich/ui';
  import { mdiCircle, mdiCircleOffOutline, mdiCircleOutline } from '@mdi/js';

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

  function title(color: LabelColor): string {
    const key = labelKey(color);
    return key ? `${LABEL_NAMES[color]} label (${key})` : `${LABEL_NAMES[color]} label`;
  }

  function pick(color: LabelColor | null): void {
    open = false;
    onchange(color);
  }
</script>

<Popover {open} onOpenChange={(v) => (open = v)} appearance="control" anchor="top">
  {#snippet trigger(popoverProps)}
    <IconButton
      {...popoverProps}
      {size}
      data-label={label ?? undefined}
      variant="ghost"
      color="secondary"
      class={label ? LABEL_TEXT[label] : ''}
      icon={label ? mdiCircle : mdiCircleOutline}
      title={hint(label ? `${LABEL_NAMES[label]} label` : 'Colour label', 'label')}
      aria-label={label ? `${LABEL_NAMES[label]} label` : 'Colour label'}
      aria-expanded={open}
      {disabled}
    />
  {/snippet}
  {#each LABEL_COLORS as color (color)}
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      class={LABEL_TEXT[color]}
      icon={mdiCircle}
      title={title(color)}
      aria-label={`${LABEL_NAMES[color]} label`}
      aria-pressed={label === color}
      onclick={() => pick(color)}
    />
  {/each}
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiCircleOffOutline}
    title="No label"
    aria-label="No label"
    aria-pressed={label === null}
    onclick={() => pick(null)}
  />
</Popover>
