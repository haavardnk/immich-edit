<script lang="ts">
  import { markText } from '#lib/browse/cullMark.ts';
  import { ADVANCE_MS, ratingHud } from '#lib/browse/ratingHud.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiStar, mdiStarOutline } from '@mdi/js';

  const STARS = [1, 2, 3, 4, 5];

  const shown = $derived(ratingHud.shown);
</script>

<div
  class="pointer-events-none absolute inset-0 z-10 flex items-center justify-center"
  role="status"
  aria-live="polite"
>
  {#if shown}
    <div
      data-testid="rating-hud"
      class="flex min-w-44 flex-col items-center gap-2 rounded-xl bg-black/70 px-4 pt-3 pb-2.5 shadow-lg backdrop-blur-sm"
    >
      {#if shown.kind === 'rating'}
        <div class="flex gap-1" aria-hidden="true">
          {#each STARS as n (n)}
            {@const lit = n <= shown.state.rating}
            <Icon
              icon={lit ? mdiStar : mdiStarOutline}
              size="24px"
              class={lit ? 'text-warning-400' : 'text-white/45'}
            />
          {/each}
        </div>
      {/if}
      <p class="text-xs text-white/70">
        <span class="font-semibold text-white">{markText(shown.kind, shown.state)}</span>
        {#if shown.advancing}· next photo{/if}
      </p>
      {#if shown.advancing}
        {#key shown.serial}
          <div class="h-0.5 w-full overflow-hidden rounded-full bg-white/15">
            <div
              data-testid="rating-hud-progress"
              class="hud-progress h-full origin-left bg-primary"
              style:animation-duration="{ADVANCE_MS}ms"
            ></div>
          </div>
        {/key}
      {/if}
    </div>
  {/if}
</div>

<style>
  .hud-progress {
    animation-name: hud-progress;
    animation-timing-function: linear;
    animation-fill-mode: forwards;
  }

  @keyframes hud-progress {
    from {
      transform: scaleX(0);
    }
    to {
      transform: scaleX(1);
    }
  }
</style>
