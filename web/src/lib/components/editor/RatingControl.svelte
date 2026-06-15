<script lang="ts">
  import StarRating from '$lib/components/StarRating.svelte';
  import FavoriteButton from '$lib/components/FavoriteButton.svelte';
  import { editor } from '$lib/stores/editor.svelte';

  const rating = $derived(editor.asset?.exifInfo?.rating ?? 0);
  const isFav = $derived(editor.asset?.isFavorite ?? false);
</script>

<div class="flex items-center">
  <StarRating {rating} onchange={(n) => void editor.setRating(n)} />
  <div class="w-px h-4 bg-white/10 mx-1.5"></div>
  <FavoriteButton isFavorite={isFav} ontoggle={() => void editor.toggleFavorite()} />
</div>
