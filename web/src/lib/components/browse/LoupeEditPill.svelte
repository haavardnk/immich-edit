<script lang="ts">
  import { commandLabel, pressHandlers } from '#lib/commands/command.ts';
  import { loupeOpenEditor } from '#lib/commands/loupe/navigation.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { Button, Tooltip } from '@immich/ui';

  let { compact = false }: { compact?: boolean } = $props();
</script>

{#snippet pill(props: Record<string, unknown>)}
  <Button
    {...props}
    size={compact ? 'small' : 'medium'}
    shape="round"
    variant="filled"
    color="primary"
    class="shrink-0 font-semibold {compact ? 'h-8 px-4' : 'h-11 px-5'}"
    leadingIcon={loupeOpenEditor.icon}
    {...pressHandlers(loupeOpenEditor)}
  >
    {commandLabel(loupeOpenEditor)}
  </Button>
{/snippet}

{#if compact}
  <Tooltip text={hint('Open in editor', 'openEditor')}>
    {#snippet child({ props })}
      {@render pill(props)}
    {/snippet}
  </Tooltip>
{:else}
  {@render pill({})}
{/if}
