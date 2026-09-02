<script lang="ts">
  import { Button, Icon } from '@immich/ui';
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

<Button
  type="button"
  size="tiny"
  variant="ghost"
  color="secondary"
  class="size-6 rounded p-0 text-dark/55 hover:bg-hairline hover:text-dark"
  {title}
  aria-label={label}
  {disabled}
  {onclick}
>
  <Icon icon={mdiRestore} size="12px" aria-hidden="true" />
</Button>
