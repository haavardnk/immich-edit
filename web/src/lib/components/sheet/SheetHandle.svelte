<script lang="ts">
  import type { Attachment } from 'svelte/attachments';

  let {
    label = 'Resize sheet',
    steppable,
    detentText,
    drag,
    onstep
  }: {
    label?: string;
    steppable: boolean;
    detentText: string;
    drag: Attachment<HTMLElement>;
    onstep: () => void;
  } = $props();

  const uid = $props.id();
  const valueId = `${uid}-detent`;
</script>

<div class="flex justify-center pt-1" {@attach drag}>
  <button
    type="button"
    data-sheet-handle
    aria-label={label}
    aria-describedby={steppable ? valueId : undefined}
    aria-hidden={steppable ? undefined : 'true'}
    tabindex={steppable ? 0 : -1}
    class="relative flex h-5 w-24 cursor-grab items-center justify-center rounded-full outline-none before:absolute before:inset-x-0 before:-inset-y-3 focus-visible:ring-2 focus-visible:ring-primary"
    onclick={onstep}
  >
    <span class="h-1 w-9 rounded-full bg-light-950/25"></span>
  </button>
  {#if steppable}
    <span id={valueId} class="sr-only">{detentText}</span>
  {/if}
</div>
