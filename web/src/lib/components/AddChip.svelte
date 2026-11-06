<script lang="ts">
  import { Button } from '@immich/ui';
  import { mdiPlus } from '@mdi/js';

  let {
    label = 'Add',
    disabled = false,
    ...rest
  }: { label?: string; disabled?: boolean } & Record<string, unknown> = $props();
</script>

<Button
  size="tiny"
  variant="ghost"
  color="secondary"
  shape="round"
  leadingIcon={mdiPlus}
  class="h-6 shrink-0 border border-dashed border-light-950/25 px-2 text-[11px] touch:h-9 touch:px-3 touch:text-sm"
  {disabled}
  {...rest}
>
  {label}
</Button>
