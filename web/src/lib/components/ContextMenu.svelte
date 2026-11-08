<script lang="ts">
  import { ContextMenu as Bits } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    trigger,
    children,
    disabled = false
  }: {
    trigger: Snippet<[Record<string, unknown>]>;
    children: Snippet;
    disabled?: boolean;
  } = $props();

  function nativeFocus(props: Record<string, unknown>): Record<string, unknown> {
    const { tabindex: _tabindex, ...rest } = props;
    return rest;
  }
</script>

<Bits.Root>
  <Bits.Trigger {disabled}>
    {#snippet child({ props })}
      {@render trigger(nativeFocus(props))}
    {/snippet}
  </Bits.Trigger>
  <Bits.Portal>
    <Bits.Content
      collisionPadding={8}
      class="z-50 min-w-44 rounded-md border border-light-950/10 bg-light-100 py-1 text-dark shadow-popover dark:ring-1 dark:ring-black/20"
    >
      {@render children()}
    </Bits.Content>
  </Bits.Portal>
</Bits.Root>
