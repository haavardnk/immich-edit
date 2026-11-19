<script lang="ts">
  import StarRating from '#lib/components/StarRating.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiStarOffOutline } from '@mdi/js';

  let {
    rating,
    onpick,
    size = 'tiny'
  }: {
    rating: number | null;
    onpick: (rating: number | null) => void;
    size?: 'tiny' | 'large';
  } = $props();
</script>

<StarRating
  size={size === 'large' ? 30 : 18}
  rating={rating ?? 0}
  mixed={rating === null}
  onchange={onpick}
/>
<IconButton
  {size}
  variant="ghost"
  color="secondary"
  icon={mdiStarOffOutline}
  title="No rating"
  aria-label="No rating"
  aria-pressed={rating === 0}
  onclick={() => onpick(null)}
/>
