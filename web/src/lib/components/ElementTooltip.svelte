<script lang="ts">
  import { Tooltip } from '@immich/ui';
  import type { ComponentProps, Snippet } from 'svelte';

  let {
    child: trigger,
    ...rest
  }: Omit<ComponentProps<typeof Tooltip>, 'child'> & {
    child: Snippet<[{ props: Record<string, unknown> }]>;
  } = $props();

  function elementProps(props: Record<string, unknown>): Record<string, unknown> {
    const { type: _type, ...element } = props;
    return element;
  }
</script>

<Tooltip {...rest}>
  {#snippet child({ props })}
    {@render trigger({ props: elementProps(props) })}
  {/snippet}
</Tooltip>
