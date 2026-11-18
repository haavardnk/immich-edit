<script lang="ts">
  import { Tooltip } from '@immich/ui';
  import { mergeProps, Toggle } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    pressed,
    onPressedChange,
    label,
    onpointerenter,
    onpointerleave,
    children
  }: {
    pressed: boolean;
    onPressedChange: (pressed: boolean) => void;
    label: string;
    onpointerenter?: () => void;
    onpointerleave?: () => void;
    children: Snippet;
  } = $props();
</script>

<Tooltip text={label}>
  {#snippet child({ props })}
    <Toggle.Root
      {...mergeProps(props, { onpointerenter, onpointerleave })}
      bind:pressed={() => pressed, onPressedChange}
      aria-label={label}
      class="group grid size-7 shrink-0 place-items-center rounded-full outline-none focus-visible:ring-2 focus-visible:ring-primary touch:size-11"
    >
      {@render children()}
    </Toggle.Root>
  {/snippet}
</Tooltip>
