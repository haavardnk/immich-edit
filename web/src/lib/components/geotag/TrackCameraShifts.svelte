<script lang="ts">
  import TrackClockFields from '#lib/components/geotag/TrackClockFields.svelte';
  import TrackDisclosure from '#lib/components/geotag/TrackDisclosure.svelte';
  import TrackExpandRow from '#lib/components/geotag/TrackExpandRow.svelte';
  import type { CameraRow } from '#lib/geotag/trackCameras.svelte.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  const AS_SHOT = 'as-shot';

  let { open = $bindable(false) }: { open?: boolean } = $props();

  const session = trackSession;
  const shifts = session.shifts;

  let shown = $state<string | null>(null);

  const at = $derived(Number.isFinite(session.log.start) ? session.log.start : Date.now());
  const only = $derived(shifts.rows.length === 1 ? shifts.rows[0] : undefined);
</script>

{#snippet fields(row: CameraRow, name: string, width?: string)}
  <TrackClockFields
    {name}
    {width}
    zone={row.zone ?? AS_SHOT}
    {at}
    choices={[{ id: AS_SHOT, label: row.asShot }]}
    onZone={(id) => shifts.setZone(row.camera.key, id === AS_SHOT ? null : id)}
    shiftMs={row.shiftMs}
    onShift={(ms) => shifts.set(row.camera.key, ms)}
  />
{/snippet}

<TrackDisclosure label="Camera time" title="Camera time" caption={shifts.status} bind:open>
  {#if only}
    {@render fields(only, only.camera.label)}
  {:else}
    {#each shifts.rows as row (row.camera.key)}
      <TrackExpandRow
        label={row.camera.label}
        meta={countOf(row.camera.count, 'photo')}
        value={row.change}
        open={shown === row.camera.key}
        onToggle={() => (shown = shown === row.camera.key ? null : row.camera.key)}
      >
        {@render fields(row, row.camera.label, 'w-52')}
      </TrackExpandRow>
    {/each}
  {/if}
</TrackDisclosure>
