<script lang="ts">
  import { Popover as Bits } from 'bits-ui';
  import type { Snippet } from 'svelte';

  type Anchor = 'top' | 'bottom';
  type Align = 'start' | 'end' | 'center';
  type Appearance = 'menu' | 'control';

  let {
    open,
    anchor = 'bottom',
    align = 'start',
    onOpenChange,
    trigger,
    children,
    appearance = 'menu',
    contentClass = ''
  }: {
    open: boolean;
    anchor?: Anchor;
    align?: Align;
    onOpenChange: (open: boolean) => void;
    trigger: Snippet<[Record<string, unknown>]>;
    children: Snippet;
    appearance?: Appearance;
    contentClass?: string;
  } = $props();

  const appearanceClass = $derived(
    appearance === 'control'
      ? 'flex h-10 items-center gap-1 overflow-visible rounded-lg border border-hairline bg-neutral-900 px-1 text-dark shadow-popover'
      : 'max-h-(--bits-popover-content-available-height) overflow-y-auto rounded-md border border-white/10 bg-neutral-900 text-dark shadow-popover ring-1 ring-black/20'
  );
</script>

<Bits.Root bind:open={() => open, onOpenChange}>
  <Bits.Trigger>
    {#snippet child({ props })}
      {@render trigger(props)}
    {/snippet}
  </Bits.Trigger>
  <Bits.Portal>
    <Bits.Content
      data-popover-content
      side={anchor}
      {align}
      sideOffset={4}
      collisionPadding={8}
      class="z-40 {appearanceClass} {contentClass}"
    >
      {@render children()}
    </Bits.Content>
  </Bits.Portal>
</Bits.Root>
