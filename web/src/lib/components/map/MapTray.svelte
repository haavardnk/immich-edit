<script lang="ts">
  import MapPhotos from '#lib/components/map/MapPhotos.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import type { MapPageState } from '#lib/map/mapPage.svelte.ts';
  import type { MapTrayState } from '#lib/map/mapTray.svelte.ts';
  import MapPill from '#lib/components/map/MapPill.svelte';
  import { Button } from '@immich/ui';

  let {
    map,
    tray,
    lift,
    hidden
  }: { map: MapPageState; tray: MapTrayState; lift: number; hidden: boolean } = $props();
</script>

{#snippet showAll()}
  <Button size="small" variant="ghost" class="h-11" onclick={() => map.showAll()}>Show all</Button>
{/snippet}

{#if tray.state === 'closed' && !hidden}
  <MapPill label={map.title} onOpen={tray.open} />
{/if}

<BottomSheet
  open={tray.state !== 'closed' && !hidden}
  onOpenChange={(open) => {
    if (!open) tray.close();
  }}
  modal={false}
  close="icon"
  closeLabel="Close photos"
  detents={[{ fraction: 0.5 }, { fraction: 0.85 }]}
  bind:detent={tray.detent}
  peek={tray.state === 'carrying'}
  title={map.title}
  subtitle={map.tab === 'view' ? map.place || undefined : undefined}
  bodyClass="flex min-h-0 flex-col p-0"
  {lift}
  data-map-sheet
  header={map.picked && map.tab === 'view' ? showAll : undefined}
>
  <MapPhotos {map} heading={false} />
</BottomSheet>
