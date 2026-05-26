<script lang="ts">
  import type { Snippet } from 'svelte';

  type Anchor = 'top' | 'bottom';
  type Align = 'start' | 'end' | 'center';

  let {
    open,
    anchor = 'bottom',
    align = 'start',
    onClose,
    trigger,
    children,
    contentClass = ''
  }: {
    open: boolean;
    anchor?: Anchor;
    align?: Align;
    onClose: () => void;
    trigger: Snippet;
    children: Snippet;
    contentClass?: string;
  } = $props();

  let root = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (!open) return;
    const onPointer = (e: PointerEvent): void => {
      if (!root) return;
      if (e.target instanceof Node && root.contains(e.target)) return;
      onClose();
    };
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        onClose();
      }
    };
    window.addEventListener('pointerdown', onPointer, true);
    window.addEventListener('keydown', onKey, true);
    return () => {
      window.removeEventListener('pointerdown', onPointer, true);
      window.removeEventListener('keydown', onKey, true);
    };
  });

  const posClass = $derived.by(() => {
    const v = anchor === 'bottom' ? 'top-full mt-1' : 'bottom-full mb-1';
    const h =
      align === 'end' ? 'right-0' : align === 'center' ? 'left-1/2 -translate-x-1/2' : 'left-0';
    return `${v} ${h}`;
  });
</script>

<div class="relative inline-flex" bind:this={root}>
  {@render trigger()}
  {#if open}
    <div
      class="absolute z-40 {posClass} bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl {contentClass}"
      role="dialog"
    >
      {@render children()}
    </div>
  {/if}
</div>
