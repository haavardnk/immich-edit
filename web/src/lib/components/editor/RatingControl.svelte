<script lang="ts">
  import StarRating from '$lib/components/StarRating.svelte';
  import FavoriteButton from '$lib/components/FavoriteButton.svelte';
  import RejectButton from '$lib/components/RejectButton.svelte';
  import { isRejected } from '$lib/reject';
  import { editor } from '$lib/stores/editor.svelte';

  const rating = $derived(editor.asset?.exifInfo?.rating ?? 0);
  const isFav = $derived(editor.asset?.isFavorite ?? false);
  const rejected = $derived(editor.asset ? isRejected(editor.asset) : false);
</script>

<div
  class="flex h-7 shrink-0 items-center gap-1 [&_button]:text-white/55 [&_button:hover]:text-white/90"
>
  <StarRating {rating} size={14} onchange={(n) => void editor.setRating(n)} />
  <FavoriteButton isFavorite={isFav} ontoggle={() => void editor.toggleFavorite()} />
  <RejectButton isRejected={rejected} ontoggle={() => void editor.toggleReject()} />
</div>
