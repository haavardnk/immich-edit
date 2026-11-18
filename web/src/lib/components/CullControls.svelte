<script lang="ts">
  import StarRating from '#lib/components/StarRating.svelte';
  import FavoriteButton from '#lib/components/FavoriteButton.svelte';
  import RejectButton from '#lib/components/RejectButton.svelte';
  import LabelPicker from '#lib/components/LabelPicker.svelte';
  import type { CullActions, CullState } from '#lib/browse/cullState.ts';

  let {
    rating,
    isFavorite,
    rejected,
    label,
    owned,
    starSize,
    actions
  }: CullState & { starSize: number; actions: CullActions } = $props();
</script>

<div class="shrink-0">
  <StarRating {rating} size={starSize} disabled={!owned} onchange={actions.rate} />
</div>
<FavoriteButton {isFavorite} disabled={!owned} ontoggle={actions.favorite} />
<RejectButton isRejected={rejected} disabled={!owned} ontoggle={actions.reject} />
<LabelPicker {label} disabled={!owned} onchange={actions.label} />
