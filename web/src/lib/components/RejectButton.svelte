<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { mdiCloseCircle, mdiCloseCircleOutline } from '@mdi/js';

  interface Props {
    isRejected: boolean;
    ontoggle: () => void;
    size?: number;
  }

  let { isRejected, ontoggle, size = 15 }: Props = $props();
</script>

<button
  type="button"
  class="p-1 rounded hover:bg-white/5 leading-none transition-colors {isRejected
    ? 'text-amber-400'
    : 'text-immich-dark-fg/40 hover:text-immich-dark-fg/70'}"
  title={isRejected ? 'Unreject (X)' : 'Reject (X)'}
  onclick={ontoggle}
>
  <Icon path={isRejected ? mdiCloseCircle : mdiCloseCircleOutline} {size} />
</button>
