<script lang="ts">
  import { hint } from '$lib/keybinds';
  import { IconButton } from '@immich/ui';
  import { mdiHeart, mdiHeartOutline } from '@mdi/js';

  interface Props {
    isFavorite: boolean;
    ontoggle: () => void;
  }

  let { isFavorite, ontoggle }: Props = $props();

  const label = $derived(
    isFavorite ? hint('Unfavorite', 'favorite') : hint('Favorite', 'favorite')
  );
</script>

<IconButton
  size="tiny"
  variant="ghost"
  color="secondary"
  class={isFavorite ? 'text-red-400' : ''}
  icon={isFavorite ? mdiHeart : mdiHeartOutline}
  title={label}
  aria-label={label}
  aria-pressed={isFavorite}
  onclick={ontoggle}
/>
