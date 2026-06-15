<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { nextRatingFromKey } from '$lib/ratingShortcuts';
  import { mdiStar, mdiStarOutline } from '@mdi/js';

  interface Props {
    rating: number;
    onchange: (rating: number | null) => void;
    size?: number;
  }

  let { rating, onchange, size = 15 }: Props = $props();

  let hover = $state<number>(0);

  function onStarClick(n: number, e: MouseEvent): void {
    e.preventDefault();
    onchange(n === rating ? null : n);
  }

  function onGroupContext(e: MouseEvent): void {
    e.preventDefault();
    onchange(null);
  }

  function onGroupKey(e: KeyboardEvent): void {
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      onchange(Math.min(5, rating + 1));
      return;
    }
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      onchange(Math.max(0, rating - 1) || null);
      return;
    }
    const next = nextRatingFromKey(e.key, rating);
    if (next !== undefined) {
      e.preventDefault();
      onchange(next);
    }
  }
</script>

<div
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
      class="p-0.5 leading-none transition-colors {active
        ? 'text-immich-dark-fg'
        : 'text-immich-dark-fg/25 hover:text-immich-dark-fg/50'} {preview ? 'opacity-70' : ''}"
      title={`${n} star${n > 1 ? 's' : ''}`}
      onmouseenter={() => (hover = n)}
      onclick={(e) => onStarClick(n, e)}
    >
      <Icon path={active ? mdiStar : mdiStarOutline} {size} />
    </button>
  {/each}
</div>
