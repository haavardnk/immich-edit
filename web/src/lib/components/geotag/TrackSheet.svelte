<script lang="ts">
  import TrackOptions from '#lib/components/geotag/TrackOptions.svelte';
  import TrackSection from '#lib/components/geotag/TrackSection.svelte';
  import TrackFitBar from '#lib/components/geotag/TrackFitBar.svelte';
  import TrackSettings from '#lib/components/geotag/TrackSettings.svelte';
  import TrackSources from '#lib/components/geotag/TrackSources.svelte';
  import TrackStrip from '#lib/components/geotag/TrackStrip.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button } from '@immich/ui';

  let { lift, hidden }: { lift: number; hidden: boolean } = $props();

  const session = trackSession;

  let detent = $state(1);
  let glance = $state(0);
  let rest = $state(0);

  const loaded = $derived(session.tracks.length > 0);
  const subtitle = $derived(
    loaded
      ? `${session.clockText} · ${session.zone.replaceAll('_', ' ')}`
      : 'Load a GPX, KML or TCX file'
  );
</script>

{#snippet footer()}
  <Button
    size="large"
    class="w-full"
    loading={session.placing}
    disabled={session.counts.placed === 0 || session.placing}
    onclick={() => void session.place()}
  >
    Place {countOf(session.counts.placed, 'photo')}
  </Button>
{/snippet}

<BottomSheet
  open={session.active && !hidden}
  onOpenChange={(open) => {
    if (!open) session.close();
  }}
  modal={false}
  close="icon"
  closeLabel="Close track"
  detents={['header', { body: loaded ? glance : rest }, { body: loaded ? glance + rest : rest }]}
  bind:detent
  title={loaded ? session.summary : 'Match track'}
  {subtitle}
  bodyClass="flex h-full min-h-0 flex-col p-0"
  {lift}
  footer={loaded ? footer : undefined}
  data-track-sheet
>
  <div class="flex min-h-0 flex-1 flex-col" data-track-panel>
    {#if loaded}
      <div class="shrink-0 border-b border-hairline pt-2" bind:offsetHeight={glance}>
        <TrackFitBar />
        <TrackStrip />
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto">
        <div class="pt-2" bind:offsetHeight={rest}>
          <TrackSection />
          <TrackSettings />
          <div class="px-4 pb-3">
            <TrackOptions />
          </div>
        </div>
      </div>
    {:else}
      <div class="min-h-0 flex-1 overflow-y-auto">
        <div bind:offsetHeight={rest}>
          <TrackSources />
        </div>
      </div>
    {/if}
  </div>
</BottomSheet>
