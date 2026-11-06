<script lang="ts">
  import { coverageOf, favoriteMark } from '#lib/browse/bulk/bulkMarks.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';

  interface Props {
    isFavorite: boolean;
    mixed?: boolean;
    size?: 'tiny' | 'small' | 'medium';
    disabled?: boolean;
    ontoggle: () => void;
  }

  let { isFavorite, mixed = false, size = 'tiny', disabled = false, ontoggle }: Props = $props();

  const mark = $derived(favoriteMark(coverageOf(isFavorite, mixed)));
  const label = $derived(hint(mark.label, 'favorite'));
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
