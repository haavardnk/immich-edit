<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { hint } from '$lib/keybinds';
  import { mdiHeart, mdiHeartOutline } from '@mdi/js';

  interface Props {
    isFavorite: boolean;
    ontoggle: () => void;
    size?: number;
  }

  let { isFavorite, ontoggle, size = 15 }: Props = $props();
</script>

<button
  type="button"
  class="p-1 rounded hover:bg-white/5 leading-none transition-colors {isFavorite
    ? 'text-red-400'
    : 'text-immich-dark-fg/40 hover:text-immich-dark-fg/70'}"
  title={isFavorite ? hint('Unfavorite', 'favorite') : hint('Favorite', 'favorite')}
  onclick={ontoggle}
>
  <Icon path={isFavorite ? mdiHeart : mdiHeartOutline} {size} />
</button>
