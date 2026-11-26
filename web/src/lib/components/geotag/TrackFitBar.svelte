<script lang="ts">
  import { clockDrift, driftText } from '#lib/metadata/clockText.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Button } from '@immich/ui';

  const session = trackSession;

  const title = $derived.by(() => {
    const [only, ...rest] = session.fits;
    if (!only) return null;
    if (rest.length > 0) return `Best fit for ${session.fits.length} cameras`;
    if (session.cameras.length === 1) return `Best fit: ${clockDrift(only.fit.correctionMs)}`;
    return `Best fit for ${only.camera.label}: ${driftText(only.fit.correctionMs)}`;
  });
</script>

{#if title}
  <div class="flex items-center gap-2 py-1 ps-4 pe-2" role="group" aria-label="Best fit">
    <p class="min-w-0 flex-1 leading-tight">
      <span class="block truncate text-sm text-light-950">
        {title}
      </span>
      <span class="block truncate text-caption text-dark/65 tabular-nums">
        Puts {session.fitMatched} of {session.fitOf} photos on the track
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
