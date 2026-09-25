<script lang="ts">
  import { hint } from '$lib/keybinds';
  import { IconButton } from '@immich/ui';
  import { mdiHeart, mdiHeartOutline } from '@mdi/js';

  interface Props {
    isFavorite: boolean;
    mixed?: boolean;
    size?: 'tiny' | 'small' | 'medium';
    disabled?: boolean;
    ontoggle: () => void;
  }

  let { isFavorite, mixed = false, size = 'tiny', disabled = false, ontoggle }: Props = $props();

  const label = $derived(
    isFavorite && !mixed ? hint('Unfavorite', 'favorite') : hint('Favorite', 'favorite')
  );
</script>

<IconButton
  {size}
  variant="ghost"
  color="secondary"
  class={mixed ? 'text-red-400/50' : isFavorite ? 'text-red-400' : ''}
  icon={isFavorite || mixed ? mdiHeart : mdiHeartOutline}
  title={label}
  aria-label={label}
  aria-pressed={mixed ? 'mixed' : isFavorite}
  {disabled}
  onclick={ontoggle}
/>
