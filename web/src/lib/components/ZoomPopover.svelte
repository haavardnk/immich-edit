<script lang="ts">
  import { hint } from '$lib/keybinds';
  import { MAX_ZOOM } from '$lib/stores/ui.svelte';
  import { Button, IconButton } from '@immich/ui';
  import { mdiFitToScreenOutline, mdiMagnifyMinusOutline, mdiMagnifyPlusOutline } from '@mdi/js';
  import Popover from './Popover.svelte';
  import RangeSlider from './editor/controls/RangeSlider.svelte';

  let {
    open,
    zoom,
    nativeZoom = null,
    onOpenChange,
    onZoom,
    onFit
  }: {
    open: boolean;
    zoom: number;
    nativeZoom?: number | null;
    onOpenChange: (open: boolean) => void;
    onZoom: (zoom: number) => void;
    onFit: () => void;
  } = $props();
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
      {...props}>{zoom}%</Button
    >
  {/snippet}
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiMagnifyMinusOutline}
    title="Zoom Out"
    aria-label="Zoom Out"
    onclick={() => onZoom(zoom - 25)}
  />
  <RangeSlider
    min={25}
    max={MAX_ZOOM}
    step={5}
    value={zoom}
    label="Zoom"
    oninput={(event: Event) => onZoom((event.currentTarget as HTMLInputElement).valueAsNumber)}
    class="mx-1 w-32"
  />
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiMagnifyPlusOutline}
    title="Zoom In"
    aria-label="Zoom In"
    onclick={() => onZoom(zoom + 25)}
  />
  <Button
    size="tiny"
    variant="ghost"
    color={nativeZoom !== null && zoom === nativeZoom ? 'primary' : 'secondary'}
    class="min-w-10 px-2 font-mono"
    title="One source pixel per screen pixel"
    disabled={nativeZoom === null}
    aria-pressed={nativeZoom !== null && zoom === nativeZoom}
    onclick={() => {
      if (nativeZoom !== null) onZoom(nativeZoom);
    }}>1:1</Button
  >
  <div class="mx-0.5 h-5 w-px bg-hairline"></div>
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiFitToScreenOutline}
    title={hint('Fit to screen', 'zoomToggle')}
    aria-label={hint('Fit to screen', 'zoomToggle')}
    onclick={onFit}
  />
</Popover>
