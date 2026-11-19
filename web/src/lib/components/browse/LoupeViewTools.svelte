<script lang="ts">
  import { commandLabel, commandTitle, runCommand } from '#lib/commands/command.ts';
  import { loupeFilmstrip } from '#lib/commands/loupe/view.ts';
  import ZoomPopover from '#lib/components/ZoomPopover.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import BarToggle from '#lib/components/editor/BarToggle.svelte';

  let {
    zoom,
    fitZoom,
    fitMode,
    onZoom,
    onFit
  }: {
    zoom: number;
    fitZoom: number;
    fitMode: boolean;
    onZoom: (value: number) => void;
    onFit: () => void;
  } = $props();

  const shown = $derived(loupeFilmstrip.pressed?.() ?? false);
</script>

<ZoomPopover
  open={ui.metaPopover === 'zoom'}
  {zoom}
  {fitZoom}
  {fitMode}
  onOpenChange={(value) => (value ? ui.openPopover('zoom') : ui.closePopover())}
  {onZoom}
  {onFit}
/>
<BarToggle
  icon={loupeFilmstrip.icon}
  pressed={shown}
  label={commandLabel(loupeFilmstrip)}
  title={commandTitle(loupeFilmstrip)}
  onclick={() => runCommand(loupeFilmstrip)}>Filmstrip</BarToggle
>
