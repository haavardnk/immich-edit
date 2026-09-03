<script lang="ts">
  import { hint } from '$lib/keybinds';
  import { nextStop, sliderToZoom, SLIDER_STEPS, zoomToSlider } from '$lib/utils/zoomLevel';
  import { Button, IconButton } from '@immich/ui';
  import { mdiFitToScreenOutline, mdiMagnifyMinusOutline, mdiMagnifyPlusOutline } from '@mdi/js';
  import Popover from './Popover.svelte';
  import RangeSlider from './editor/controls/RangeSlider.svelte';

  let {
    open,
    zoom,
    fitZoom,
    fitMode,
    onOpenChange,
    onZoom,
    onFit
  }: {
    open: boolean;
    zoom: number;
    fitZoom: number;
    fitMode: boolean;
    onOpenChange: (open: boolean) => void;
    onZoom: (zoom: number) => void;
    onFit: () => void;
  } = $props();

  const percent = $derived(`${Math.round(zoom)}%`);
  const atNative = $derived(!fitMode && Math.round(zoom) === 100);
</script>

<Popover {open} anchor="top" align="end" {onOpenChange} appearance="control">
  {#snippet trigger(props)}
    <Button
      size="tiny"
      variant="ghost"
      color={open ? 'primary' : 'secondary'}
      class="min-w-12 px-1 font-mono tabular-nums text-white/70 hover:text-white"
      title="Zoom"
      aria-label="Zoom"
      {...props}>{fitMode ? 'Fit' : percent}</Button
    >
  {/snippet}
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiMagnifyMinusOutline}
    title="Zoom Out"
    aria-label="Zoom Out"
    onclick={() => onZoom(nextStop(zoom, -1, fitZoom))}
  />
  <RangeSlider
    min={0}
    max={SLIDER_STEPS}
    step={1}
    value={zoomToSlider(zoom, fitZoom)}
    label="Zoom"
    valueText={percent}
    oninput={(event: Event) =>
      onZoom(sliderToZoom((event.currentTarget as HTMLInputElement).valueAsNumber, fitZoom))}
    class="mx-1 w-32"
  />
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiMagnifyPlusOutline}
    title="Zoom In"
    aria-label="Zoom In"
    onclick={() => onZoom(nextStop(zoom, 1, fitZoom))}
  />
  <Button
    size="tiny"
    variant="ghost"
    color={atNative ? 'primary' : 'secondary'}
    class="min-w-10 px-2 font-mono"
    title="One source pixel per screen pixel"
    aria-pressed={atNative}
    onclick={() => onZoom(100)}>1:1</Button
  >
  <div class="mx-0.5 h-5 w-px bg-hairline"></div>
  <IconButton
    size="tiny"
    variant="ghost"
    color={fitMode ? 'primary' : 'secondary'}
    icon={mdiFitToScreenOutline}
    title={hint('Fit to screen', 'zoomToggle')}
    aria-label={hint('Fit to screen', 'zoomToggle')}
    onclick={onFit}
  />
</Popover>
