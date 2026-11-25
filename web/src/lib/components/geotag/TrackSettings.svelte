<script lang="ts">
  import { correctionText, parseCorrection } from '#lib/geotag/clockText.ts';
  import { offsetLabel, zoneOffsetMinutes } from '#lib/geotag/photoTime.ts';
  import SearchableSelect from '#lib/components/SearchableSelect.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import TrackCameraPick from '#lib/components/geotag/TrackCameraPick.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import { TOLERANCES, trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiMinus, mdiPlus } from '@mdi/js';

  const STEP = 60_000;
  const HOUR = 60 * STEP;

  const session = trackSession;
  const zones = Intl.supportedValuesOf('timeZone');

  let typed = $state<string | null>(null);
  let within = $state(false);

  const reference = $derived(Number.isFinite(session.log.start) ? session.log.start : Date.now());

  function minutesLabel(minutes: number): string {
    return minutes === 60 ? '1 h' : `${minutes} min`;
  }

  function zoneLabel(zone: string): string {
    return `${zone.replaceAll('_', ' ')} · ${offsetLabel(zoneOffsetMinutes(zone, reference))}`;
  }

  function shift(by: number): void {
    session.correctionMs += by;
  }

  function commit(): void {
    if (typed === null) return;
    const parsed = parseCorrection(typed);
    if (parsed !== null) session.correctionMs = parsed;
    typed = null;
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
      <div class="w-48 shrink-0 touch:w-full">
        <SearchableSelect
          compact
          color="neutral"
          multiple={false}
          options={zones}
          selected={[session.zone]}
          getId={(zone) => zone}
          getLabel={zoneLabel}
          label="Camera time zone"
          onSelectedChange={(next) => {
            const zone = next[0];
            if (zone) session.pickZone(zone);
          }}
        />
      </div>
    </div>
  {/if}
  <TrackCameraPick />
  <div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
    <div class="min-w-0 flex-1 text-sm text-light-950">Correction</div>
    <div
      class="flex w-60 shrink-0 items-center rounded-lg bg-light-100 ring-1 ring-hairline touch:w-full"
    >
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-8 px-2 whitespace-nowrap touch:h-11"
        aria-label="One hour earlier"
        onclick={() => shift(-HOUR)}
      >
        −1 h
      </Button>
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:size-11"
        icon={mdiMinus}
        title="One minute earlier"
        aria-label="One minute earlier"
        onclick={() => shift(-STEP)}
      />
      <TextInput
        compact
        class="w-full font-semibold tabular-nums [&_input]:text-center"
        aria-label="Clock correction"
        value={typed ?? correctionText(session.correctionMs)}
        oninput={(event) => (typed = event.currentTarget.value)}
        onblur={commit}
        onkeydown={(event) => {
          if (event.key === 'Enter') commit();
        }}
      />
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:size-11"
        icon={mdiPlus}
        title="One minute later"
        aria-label="One minute later"
        onclick={() => shift(STEP)}
      />
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-8 px-2 whitespace-nowrap touch:h-11"
        aria-label="One hour later"
        onclick={() => shift(HOUR)}
      >
        +1 h
      </Button>
    </div>
  </div>
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
