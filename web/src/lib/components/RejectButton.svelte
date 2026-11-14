<script lang="ts">
  import { coverageOf, rejectMark } from '#lib/browse/bulk/bulkMarks.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';

  interface Props {
    isRejected: boolean;
    mixed?: boolean;
    size?: 'tiny' | 'small' | 'medium';
    disabled?: boolean;
    ontoggle: () => void;
  }

  let { isRejected, mixed = false, size = 'tiny', disabled = false, ontoggle }: Props = $props();

  const mark = $derived(rejectMark(coverageOf(isRejected, mixed)));
  const label = $derived(hint(mark.label, 'reject'));
</script>

<IconButton
  {size}
  variant="ghost"
  color="secondary"
  class={mark.tone}
  icon={mark.icon}
  title={label}
  aria-label={label}
  aria-pressed={mark.pressed}
  {disabled}
  onclick={ontoggle}
/>
