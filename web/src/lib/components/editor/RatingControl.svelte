<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { mdiHeart, mdiHeartOutline, mdiStar, mdiStarOutline } from '@mdi/js';

  const rating = $derived(editor.asset?.exifInfo?.rating ?? 0);
  const isFav = $derived(editor.asset?.isFavorite ?? false);
  let hover = $state<number>(0);
  let groupEl = $state<HTMLDivElement | null>(null);

  function setRating(n: number): void {
    void editor.setRating(n === 0 ? null : n);
  }

  function onStarClick(n: number, e: MouseEvent): void {
    e.preventDefault();
    if (rating === n) {
      setRating(0);
    } else {
      setRating(n);
    }
  }

  function onGroupContext(e: MouseEvent): void {
    e.preventDefault();
    setRating(0);
  }

  function onGroupKey(e: KeyboardEvent): void {
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      setRating(Math.min(5, rating + 1));
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      setRating(Math.max(0, rating - 1));
    } else if (e.key >= '0' && e.key <= '5') {
      e.preventDefault();
      setRating(Number(e.key));
    }
  }
</script>

<div class="flex items-center">
  <div
    bind:this={groupEl}
    role="radiogroup"
    aria-label="Rating"
    tabindex="0"
    class="flex items-center px-1 rounded focus:outline-none focus:ring-1 focus:ring-white/20"
    oncontextmenu={onGroupContext}
    onkeydown={onGroupKey}
    onmouseleave={() => (hover = 0)}
  >
    {#each [1, 2, 3, 4, 5] as n (n)}
      {@const active = hover > 0 ? n <= hover : n <= rating}
      {@const preview = hover > 0 && n <= hover && n > rating}
      <button
        type="button"
        role="radio"
        aria-checked={n === rating}
        tabindex="-1"
        class="p-0.5 leading-none transition-colors {active ? 'text-amber-400' : 'text-immich-dark-fg/25 hover:text-immich-dark-fg/50'} {preview ? 'opacity-70' : ''}"
        title={`${n} star${n > 1 ? 's' : ''}`}
        onmouseenter={() => (hover = n)}
        onclick={(e) => onStarClick(n, e)}
      >
        <Icon path={active ? mdiStar : mdiStarOutline} size={15} />
      </button>
    {/each}
  </div>
  <div class="w-px h-4 bg-white/10 mx-1.5"></div>
  <button
    type="button"
    class="p-1 rounded hover:bg-white/5 leading-none transition-colors {isFav ? 'text-red-400' : 'text-immich-dark-fg/40 hover:text-immich-dark-fg/70'}"
    title={isFav ? 'Unfavorite (F)' : 'Favorite (F)'}
    onclick={() => void editor.toggleFavorite()}
  >
    <Icon path={isFav ? mdiHeart : mdiHeartOutline} size={15} />
  </button>
</div>
