<script lang="ts">
  import TrackCameraShifts from '#lib/components/geotag/TrackCameraShifts.svelte';
  import TrackDisclosureHeader from '#lib/components/geotag/TrackDisclosureHeader.svelte';
  import TrackOptions from '#lib/components/geotag/TrackOptions.svelte';
  import TrackPhotoList from '#lib/components/geotag/TrackPhotoList.svelte';
  import TrackPlaceButton from '#lib/components/geotag/TrackPlaceButton.svelte';
  import TrackSection from '#lib/components/geotag/TrackSection.svelte';
  import TrackSources from '#lib/components/geotag/TrackSources.svelte';
  import TrackStrip from '#lib/components/geotag/TrackStrip.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { CONTENT_CAP } from '#lib/components/sheet/sheetSnap.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';

  let { lift, hidden }: { lift: number; hidden: boolean } = $props();

  const session = trackSession;

  let detent = $state(1);
  let glance = $state(0);
  let rest = $state(0);
  let shifting = $state(false);

  const loaded = $derived(session.tracks.length > 0);
  const subtitle = $derived(loaded ? undefined : 'Load a GPX, KML or TCX file');
</script>

{#snippet footer()}
  <div class="flex w-full flex-col gap-3">
    {#if detent === 2}
      <TrackOptions />
    {/if}
    <TrackPlaceButton size="large" class="w-full" />
  </div>
{/snippet}

<BottomSheet
  open={session.active && !hidden}
  onOpenChange={(open) => {
    if (!open) session.close();
  }}
  modal={false}
  close="icon"
  closeLabel="Close track"
  detents={[
    'header',
    { body: loaded ? glance : rest },
    loaded ? { fraction: CONTENT_CAP } : { body: rest }
  ]}
  bind:detent
  collapseTo={loaded ? 1 : undefined}
  title={loaded ? session.summary : 'Match track'}
  {subtitle}
  bodyClass="flex h-full min-h-0 flex-col p-0"
  {lift}
  footer={loaded ? footer : undefined}
  data-track-sheet
>
  <div class="flex min-h-0 flex-1 flex-col" data-track-panel>
    {#if loaded}
      <div
        class={['shrink-0 border-t border-hairline', detent === 2 && 'hidden']}
        bind:offsetHeight={
          null,
          (height) => {
            if (height) glance = height;
          }
        }
      >
        <TrackDisclosureHeader
          title="Camera time"
          caption={session.shifts.status}
          open={false}
          onclick={() => {
            shifting = true;
            detent = 2;
          }}
        />
        <TrackStrip />
      </div>
      <div class="flex min-h-0 flex-1 flex-col overflow-y-auto border-t border-hairline">
        <div class="shrink-0" bind:offsetHeight={rest}>
          <TrackSection />
          <TrackCameraShifts bind:open={shifting} />
        </div>
        <div class={detent === 2 ? 'contents' : 'hidden'}>
          <TrackPhotoList />
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
