<script lang="ts">
  import { commandLabel, commandTitle, pressHandlers } from '#lib/commands/command.ts';
  import { loupeAutoAdvance } from '#lib/commands/loupe/view.ts';
  import { Button } from '@immich/ui';

  let { class: className = '' }: { class?: string } = $props();

  const on = $derived(loupeAutoAdvance.pressed?.() ?? false);
</script>

<Button
  size="tiny"
  shape="round"
  variant="ghost"
  color={on ? 'primary' : 'secondary'}
  class="h-7 shrink-0 px-2.5 {on ? 'bg-primary/15 hover:bg-primary/25' : ''} {className}"
  leadingIcon={loupeAutoAdvance.icon}
  title={commandTitle(loupeAutoAdvance)}
  aria-pressed={on}
  {...pressHandlers(loupeAutoAdvance)}
>
  <span class="sr-only @lg:not-sr-only">{commandLabel(loupeAutoAdvance)}</span>
</Button>
