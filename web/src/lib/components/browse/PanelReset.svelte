<script lang="ts">
  import { layout } from '#lib/device/layout.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiRestore } from '@mdi/js';

  let { label, disabled, onclick }: { label: string; disabled: boolean; onclick: () => void } =
    $props();
</script>

<Button
  size="tiny"
  variant="ghost"
  color={layout.mode === 'touch' ? 'primary' : 'secondary'}
  class="ml-auto h-8 shrink-0 bg-transparent hover:bg-light-950/6 touch:text-label"
  leadingIcon={mdiRestore}
  aria-label={label}
  {disabled}
  {onclick}
>
  Reset
</Button>
