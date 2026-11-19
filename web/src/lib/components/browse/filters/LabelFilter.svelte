<script lang="ts">
  import Chip from '#lib/components/chips/Chip.svelte';
  import IconToggle from '#lib/components/chips/IconToggle.svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { LABEL_COLORS, LABEL_NAMES, LABEL_TEXT } from '#lib/stores/labels.ts';
  import { mdiCircleOffOutline } from '@mdi/js';

  const label = $derived(browseControls.filters.label);
</script>

<div class="flex items-center gap-2">
  <div role="group" aria-label="Label" class="-ml-1 flex touch:-ml-2.5">
    {#each LABEL_COLORS as color (color)}
      <IconToggle
        pressed={label === color}
        onPressedChange={(pressed) => (browseControls.filters.label = pressed ? color : 'any')}
        label="{LABEL_NAMES[color]} label"
      >
        <span
          class="size-3.5 rounded-full bg-current ring-offset-2 ring-offset-light group-aria-pressed:ring-[1.5px] group-aria-pressed:ring-light-950 touch:size-5.5 touch:group-aria-pressed:ring-2 {LABEL_TEXT[
            color
          ]}"
          aria-hidden="true"
        ></span>
      </IconToggle>
    {/each}
  </div>
  <span class="flex-1"></span>
  <Chip
    pressed={label === 'none'}
    onPressedChange={(pressed) => (browseControls.filters.label = pressed ? 'none' : 'any')}
    icon={mdiCircleOffOutline}
  >
    No label
  </Chip>
</div>
