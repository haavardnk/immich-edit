<script lang="ts">
  import { sameTagColor, tagSwatches } from '#lib/browse/tagColor.ts';
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import { Icon } from '@immich/ui';
  import { mdiCancel } from '@mdi/js';
  import { RadioGroup } from 'bits-ui';
  import { untrack } from 'svelte';

  const NONE = 'none';

  let {
    value,
    onChange,
    ref = $bindable(null)
  }: {
    value: string | null;
    onChange: (color: string | null) => void;
    ref?: HTMLElement | null;
  } = $props();

  const swatches = tagSwatches(untrack(() => value));
  const selected = $derived(
    swatches.find((swatch) => sameTagColor(swatch.value, value))?.value ?? NONE
  );

  const SWATCH =
    'relative size-6 shrink-0 rounded-full ring-1 ring-hairline ring-offset-2 ring-offset-light-50 outline-none transition-shadow hover:ring-light-950/35 focus-visible:ring-2 focus-visible:ring-primary aria-checked:ring-2 aria-checked:ring-dark touch:size-8 touch:before:absolute touch:before:-inset-1.5';
</script>

<RadioGroup.Root
  bind:ref
  value={selected}
  onValueChange={(next) => onChange(next === NONE ? null : next)}
  orientation="horizontal"
  aria-label="Color"
  class="flex flex-wrap items-center gap-2.5 touch:gap-3"
>
  <ElementTooltip text="No color">
    {#snippet child({ props })}
      <RadioGroup.Item
        {...props}
        value={NONE}
        aria-label="No color"
        class="{SWATCH} grid place-items-center bg-light-200 text-dark/65"
      >
        <Icon icon={mdiCancel} size="14px" aria-hidden="true" />
      </RadioGroup.Item>
    {/snippet}
  </ElementTooltip>
  {#each swatches as swatch (swatch.value)}
    <ElementTooltip text={swatch.label}>
      {#snippet child({ props })}
        <RadioGroup.Item
          {...props}
          value={swatch.value}
          aria-label={swatch.label}
          class={SWATCH}
          style="background-color: {swatch.value}"
        />
      {/snippet}
    </ElementTooltip>
  {/each}
</RadioGroup.Root>
