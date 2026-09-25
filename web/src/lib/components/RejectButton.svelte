<script lang="ts">
  import { hint } from '$lib/keybinds';
  import { IconButton } from '@immich/ui';
  import { mdiCloseCircle, mdiCloseCircleOutline } from '@mdi/js';

  interface Props {
    isRejected: boolean;
    mixed?: boolean;
    size?: 'tiny' | 'small' | 'medium';
    disabled?: boolean;
    ontoggle: () => void;
  }

  let { isRejected, mixed = false, size = 'tiny', disabled = false, ontoggle }: Props = $props();

  const label = $derived(
    isRejected && !mixed ? hint('Unreject', 'reject') : hint('Reject', 'reject')
  );
</script>

<IconButton
  {size}
  variant="ghost"
  color="secondary"
  class={mixed ? 'text-amber-400/50' : isRejected ? 'text-amber-400' : ''}
  icon={isRejected || mixed ? mdiCloseCircle : mdiCloseCircleOutline}
  title={label}
  aria-label={label}
  aria-pressed={mixed ? 'mixed' : isRejected}
  {disabled}
  onclick={ontoggle}
/>
