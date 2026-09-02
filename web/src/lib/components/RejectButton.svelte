<script lang="ts">
  import { hint } from '$lib/keybinds';
  import { IconButton } from '@immich/ui';
  import { mdiCloseCircle, mdiCloseCircleOutline } from '@mdi/js';

  interface Props {
    isRejected: boolean;
    ontoggle: () => void;
  }

  let { isRejected, ontoggle }: Props = $props();

  const label = $derived(isRejected ? hint('Unreject', 'reject') : hint('Reject', 'reject'));
</script>

<IconButton
  size="tiny"
  variant="ghost"
  color="secondary"
  class={isRejected ? 'text-amber-400' : ''}
  icon={isRejected ? mdiCloseCircle : mdiCloseCircleOutline}
  title={label}
  aria-label={label}
  aria-pressed={isRejected}
  onclick={ontoggle}
/>
