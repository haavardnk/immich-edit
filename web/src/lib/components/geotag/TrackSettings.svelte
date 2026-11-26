<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import CameraPick from '#lib/components/metadata/CameraPick.svelte';
  import ClockStepper from '#lib/components/metadata/ClockStepper.svelte';
  import ZoneSelect from '#lib/components/metadata/ZoneSelect.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { TOLERANCES, trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Button } from '@immich/ui';

  const session = trackSession;

  let within = $state(false);

  const reference = $derived(Number.isFinite(session.log.start) ? session.log.start : Date.now());

  function minutesLabel(minutes: number): string {
    return minutes === 60 ? '1 h' : `${minutes} min`;
  }
</script>

<section class="flex flex-col gap-3 px-4 py-3" aria-label="Camera clock">
  {#if session.unzoned > 0}
    <div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
      <div class="min-w-0 flex-1 text-sm text-light-950">
        Time zone
        <span class="block text-caption text-dark/65">
          {session.unzoned} photos without one{session.zoneFrom === 'track'
            ? ' · from the track'
            : ''}
        </span>
      </div>
      <ZoneSelect
        label="Time zone"
        value={session.zone}
        at={reference}
        onPick={(zone) => session.pickZone(zone)}
      />
    </div>
  {/if}
  <CameraPick
    cameras={session.cameras}
    chosen={session.camera}
    detail={(key) => session.shiftTextOf(key)}
    onChoose={(key) => session.clocks.choose(key)}
  />
  <ClockStepper
    label="Shift by"
    name="Shift by"
    value={session.correctionMs}
    onChange={(ms) => (session.correctionMs = ms)}
  />
  <div class="flex items-center gap-3">
    <span class="min-w-0 flex-1 text-sm text-light-950"> Match within </span>
    <Popover
      open={within}
      onOpenChange={(open) => (within = open)}
      title="Match within"
      subtitle="How far from a track point a photo may be"
    >
      {#snippet trigger(props)}
        <Button
          {...props}
          size="small"
          variant="ghost"
          color="secondary"
          class="h-8 bg-light-100 ring-1 ring-hairline touch:h-11"
          aria-label="Match within {minutesLabel(session.toleranceMin)}"
        >
          {minutesLabel(session.toleranceMin)}
        </Button>
      {/snippet}
      {#each TOLERANCES as minutes (minutes)}
        <MenuRow
          label={minutesLabel(minutes)}
          pressed={session.toleranceMin === minutes}
          onclick={() => {
            session.toleranceMin = minutes;
            within = false;
          }}
        />
      {/each}
    </Popover>
  </div>
</section>
