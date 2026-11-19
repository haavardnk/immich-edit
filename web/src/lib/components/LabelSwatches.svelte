<script lang="ts">
  import {
    LABEL_COLORS,
    LABEL_NAMES,
    LABEL_TEXT,
    labelKey,
    type LabelColor
  } from '#lib/stores/labels.ts';
  import { IconButton } from '@immich/ui';
  import { mdiCircle, mdiCircleOffOutline } from '@mdi/js';

  let {
    label,
    onpick,
    size = 'tiny'
  }: {
    label: LabelColor | null;
    onpick: (color: LabelColor | null) => void;
    size?: 'tiny' | 'large';
  } = $props();

  function title(color: LabelColor): string {
    const key = labelKey(color);
    return key ? `${LABEL_NAMES[color]} label (${key})` : `${LABEL_NAMES[color]} label`;
  }
</script>

{#each LABEL_COLORS as color (color)}
  <IconButton
    {size}
    variant="ghost"
    color="secondary"
    class={LABEL_TEXT[color]}
    icon={mdiCircle}
    title={title(color)}
    aria-label={`${LABEL_NAMES[color]} label`}
    aria-pressed={label === color}
    onclick={() => onpick(color)}
  />
{/each}
<IconButton
  {size}
  variant="ghost"
  color="secondary"
  icon={mdiCircleOffOutline}
  title="No label"
  aria-label="No label"
  aria-pressed={label === null}
  onclick={() => onpick(null)}
/>
