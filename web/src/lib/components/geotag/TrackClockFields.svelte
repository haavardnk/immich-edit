<script lang="ts">
  import ClockStepper from '#lib/components/metadata/ClockStepper.svelte';
  import ZoneSelect from '#lib/components/metadata/ZoneSelect.svelte';

  interface Choice {
    id: string;
    label: string;
  }

  let {
    name,
    zone,
    zones = true,
    at,
    choices,
    onZone,
    shiftMs,
    width = 'w-60',
    onShift
  }: {
    name: string;
    zone: string;
    zones?: boolean;
    at: number;
    choices: readonly Choice[];
    onZone: (id: string) => void;
    shiftMs: number;
    width?: string;
    onShift: (ms: number) => void;
  } = $props();
</script>

{#if zones}
  <div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
    <span class="min-w-0 flex-1 text-sm whitespace-nowrap text-light-950">Time zone</span>
    <ZoneSelect
      label="{name} time zone"
      value={zone}
      {at}
      {choices}
      class={width}
      onPick={onZone}
    />
  </div>
{/if}
<ClockStepper label="Shift by" name="Shift {name} by" value={shiftMs} {width} onChange={onShift} />
