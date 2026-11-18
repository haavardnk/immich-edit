<script lang="ts">
  import { Dialog } from 'bits-ui';

  let { tone = 'dim', leaving = false }: { tone?: 'dim' | 'light'; leaving?: boolean } = $props();
</script>

<Dialog.Overlay>
  {#snippet child({ props })}
    {@const ending = 'data-ending-style' in props}
    <div
      {...props}
      data-leaving={leaving || ending ? '' : undefined}
      class="fixed inset-0 z-50 {tone === 'light'
        ? 'bg-black/20'
        : 'bg-black/60'} starting:opacity-0 data-leaving:opacity-0 motion-safe:transition-opacity motion-safe:duration-slow motion-safe:ease-standard motion-safe:data-leaving:duration-base motion-safe:data-leaving:ease-exit"
      style:pointer-events={ending ? 'none' : 'auto'}
    ></div>
  {/snippet}
</Dialog.Overlay>
