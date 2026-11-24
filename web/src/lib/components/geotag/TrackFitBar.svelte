<script lang="ts">
  import { clockDrift } from '#lib/geotag/clockText.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Button } from '@immich/ui';

  const session = trackSession;

  const fit = $derived(
    session.fit && session.fit.correctionMs !== session.correctionMs ? session.fit : null
  );
</script>

{#if fit}
  <div class="flex items-center gap-2 py-1 ps-4 pe-2" role="group" aria-label="Best fit">
    <p class="min-w-0 flex-1 leading-tight">
      <span class="block truncate text-sm text-light-950">
        Best fit: {clockDrift(fit.correctionMs)}
      </span>
      <span class="block truncate text-caption text-dark/65 tabular-nums">
        Puts {fit.matched} of {session.photos.length} photos on the track
      </span>
    </p>
    <Button
      size="small"
      variant="ghost"
      class="touch:h-11"
      aria-label="Use best fit"
      onclick={session.useFit}
    >
      Use
    </Button>
  </div>
{/if}
