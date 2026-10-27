<script lang="ts">
  import { IconButton } from '@immich/ui';
  import { mdiRestore } from '@mdi/js';

  let {
    title,
    label = title,
    disabled = false,
    onclick
  }: {
    title: string;
    label?: string;
    disabled?: boolean;
    onclick: (e: MouseEvent) => void;
  } = $props();
</script>

<IconButton
  size="tiny"
  variant="ghost"
  color="secondary"
  icon={mdiRestore}
  {title}
  aria-label={label}
  {disabled}
  {onclick}
/>
