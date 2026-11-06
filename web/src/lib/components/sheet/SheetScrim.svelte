<script lang="ts">
  import { Dialog } from 'bits-ui';

  let { tone = 'dim' }: { tone?: 'dim' | 'light' } = $props();
</script>

<Dialog.Overlay
  class="fixed inset-0 z-50 {tone === 'light'
    ? 'bg-black/20'
    : 'bg-black/60'} motion-safe:transition-opacity motion-safe:duration-300 starting:opacity-0"
/>
