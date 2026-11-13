<script lang="ts">
  import MenuItem from '#lib/components/MenuItem.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import {
    MASK_OVERLAY_MODES,
    maskOverlay,
    type MaskOverlayMode
  } from '#lib/stores/maskOverlay.svelte.ts';
  import { MASK_OVERLAY_COLORS } from '#lib/theme/maskOverlay.ts';
  import { IconButton, Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import {
    mdiChevronDown,
    mdiCircle,
    mdiCircleHalfFull,
    mdiCircleOutline,
    mdiEyeOutline
  } from '@mdi/js';

  const MODE_ICONS: Record<MaskOverlayMode, string> = {
    color: mdiCircle,
    colorOnBw: mdiCircleHalfFull,
    whiteOnBlack: mdiCircleOutline
  };
  const heading = 'px-3 pt-1.5 pb-1 text-[10px] tracking-wider text-dark/65 uppercase touch:px-4';

  let open = $state(false);
</script>

<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  contentClass="w-60 py-1"
  title="Mask overlay"
  tooltip="Overlay display and color"
>
  {#snippet trigger(props)}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="w-5 bg-transparent hover:bg-light-950/6"
      icon={mdiChevronDown}
      title=""
      aria-label="Mask overlay options"
      {...props}
    />
  {/snippet}
  <div class="flex flex-col">
    <span class={heading}>Overlay · {keysFor('maskOverlayMode')} cycles</span>
    {#each MASK_OVERLAY_MODES as entry (entry.id)}
      <MenuItem
        icon={MODE_ICONS[entry.id]}
        label={entry.label}
        pressed={maskOverlay.mode === entry.id}
        onclick={() => maskOverlay.setMode(entry.id)}
      />
    {/each}
    <span class={heading}>Color</span>
    <RadioGroup.Root
      value={maskOverlay.color}
      onValueChange={(hex) => void maskOverlay.setColor(hex)}
      orientation="horizontal"
      aria-label="Overlay color"
      class="flex gap-2 px-3 pt-1 pb-2 touch:px-4"
    >
      {#each MASK_OVERLAY_COLORS as swatch (swatch.hex)}
        <Tooltip text={swatch.name}>
          {#snippet child({ props })}
            <RadioGroup.Item
              {...props}
              value={swatch.hex}
              aria-label={swatch.name}
              class="size-5 rounded-full ring-1 ring-hairline ring-offset-2 ring-offset-light-100 outline-none transition-shadow hover:ring-light-950/35 focus-visible:ring-2 focus-visible:ring-primary aria-checked:ring-2 aria-checked:ring-dark touch:size-8"
              style="background-color: {swatch.hex}"
            />
          {/snippet}
        </Tooltip>
      {/each}
    </RadioGroup.Root>
    <div class="border-t border-hairline pt-1">
      <MenuItem
        icon={mdiEyeOutline}
        label="Show overlay"
        keys={keysFor('maskOverlay')}
        pressed={editor.maskOverlayVisible}
        onclick={editor.toggleMaskOverlay}
      />
    </div>
  </div>
</Popover>
