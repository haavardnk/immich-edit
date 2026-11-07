<script lang="ts">
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import { IconButton, type IconButtonProps } from '@immich/ui';

  let { tooltip, ...rest }: Extract<IconButtonProps, { href: string }> & { tooltip?: string } =
    $props();
</script>

<ElementTooltip text={tooltip ?? rest['aria-label']}>
  {#snippet child({ props })}
    <IconButton {...props} {...rest} title="" />
  {/snippet}
</ElementTooltip>
