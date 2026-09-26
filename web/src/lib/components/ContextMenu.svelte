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
</script>

<Bits.Root>
  <Bits.Trigger {disabled}>
    {#snippet child({ props })}
      {@render trigger(props)}
    {/snippet}
  </Bits.Trigger>
  <Bits.Portal>
    <Bits.Content
      collisionPadding={8}
      class="z-50 min-w-44 rounded-md border border-white/10 bg-neutral-900 py-1 text-dark shadow-popover ring-1 ring-black/20"
    >
      {@render children()}
    </Bits.Content>
  </Bits.Portal>
</Bits.Root>
