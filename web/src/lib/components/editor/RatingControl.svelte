<script lang="ts">
  import StarRating from '$lib/components/StarRating.svelte';
  import FavoriteButton from '$lib/components/FavoriteButton.svelte';
  import RejectButton from '$lib/components/RejectButton.svelte';
  import LabelPicker from '$lib/components/LabelPicker.svelte';
  import { isRejected } from '$lib/browse/reject';
  import { ownsAsset } from '$lib/browse/ownership';
  import { labelOf } from '$lib/stores/labels';
  import { editor } from '$lib/stores/editor.svelte';

  const rating = $derived(editor.asset?.exifInfo?.rating ?? 0);
  const isFav = $derived(editor.asset?.isFavorite ?? false);
  const rejected = $derived(editor.asset ? isRejected(editor.asset) : false);
  const label = $derived(editor.asset ? labelOf(editor.asset) : null);
  const owned = $derived(editor.asset ? ownsAsset(editor.asset) : true);
</script>

<div
  class="flex h-7 shrink-0 items-center gap-1 [&_button:not([data-label])]:text-white/55 [&_button:not([data-label]):hover]:text-white/90"
>
  <StarRating {rating} size={14} disabled={!owned} onchange={(n) => void editor.setRating(n)} />
  <FavoriteButton
    isFavorite={isFav}
    disabled={!owned}
    ontoggle={() => void editor.toggleFavorite()}
  />
  <RejectButton
    isRejected={rejected}
    disabled={!owned}
    ontoggle={() => void editor.toggleReject()}
  />
  <LabelPicker {label} disabled={!owned} onchange={(color) => void editor.setLabel(color)} />
</div>
