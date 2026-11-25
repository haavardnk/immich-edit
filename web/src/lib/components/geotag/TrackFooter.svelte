<script lang="ts">
  import TrackOptions from '#lib/components/geotag/TrackOptions.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button } from '@immich/ui';

  let { framed = true }: { framed?: boolean } = $props();

  const session = trackSession;
</script>

<footer
  class="flex w-full shrink-0 flex-col gap-3 {framed
    ? 'border-t border-hairline bg-light-50 px-4 py-3'
    : ''}"
>
  <TrackOptions />
  <div class="flex items-center justify-end gap-2">
    <Button
      size="small"
      variant="ghost"
      color="secondary"
      class="touch:h-11"
      onclick={session.close}
    >
      Cancel
    </Button>
    <Button
      size="small"
      class="shrink-0 whitespace-nowrap touch:h-11"
      loading={session.placing}
      disabled={session.counts.placed === 0 || session.placing}
      onclick={() => void session.place()}
    >
      Place {countOf(session.counts.placed, 'photo')}
    </Button>
  </div>
</footer>
