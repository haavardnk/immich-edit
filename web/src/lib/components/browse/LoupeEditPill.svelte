<script lang="ts">
  import { commandLabel, pressHandlers } from '#lib/commands/command.ts';
  import { loupeOpenEditor } from '#lib/commands/loupe/navigation.ts';
  import { Button } from '@immich/ui';
</script>

<Button
  size="medium"
  shape="round"
  variant="filled"
  color="primary"
  class="h-11 shrink-0 px-5 font-semibold"
  leadingIcon={loupeOpenEditor.icon}
  {...pressHandlers(loupeOpenEditor)}
>
  {commandLabel(loupeOpenEditor)}
</Button>
