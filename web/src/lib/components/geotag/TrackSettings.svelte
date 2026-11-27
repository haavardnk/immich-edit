<script lang="ts">
  import CameraPick from '#lib/components/metadata/CameraPick.svelte';
  import ClockStepper from '#lib/components/metadata/ClockStepper.svelte';
  import ZoneSelect from '#lib/components/metadata/ZoneSelect.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';

  const session = trackSession;

  const reference = $derived(Number.isFinite(session.log.start) ? session.log.start : Date.now());
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
</section>
