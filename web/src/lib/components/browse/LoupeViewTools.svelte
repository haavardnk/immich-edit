<script lang="ts">
  import { commandLabel, commandTitle, pressHandlers } from '#lib/commands/command.ts';
  import { loupeFilmstrip } from '#lib/commands/loupe/view.ts';
  import ZoomPopover from '#lib/components/ZoomPopover.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { IconButton } from '@immich/ui';

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
<IconButton
  size="tiny"
  variant="ghost"
  color={shown ? 'primary' : 'secondary'}
  class="touch:size-11 {shown ? 'touch:bg-primary/15' : ''}"
  icon={loupeFilmstrip.icon}
  title={commandTitle(loupeFilmstrip)}
  aria-label={commandLabel(loupeFilmstrip)}
  aria-pressed={shown}
  {...pressHandlers(loupeFilmstrip)}
/>
