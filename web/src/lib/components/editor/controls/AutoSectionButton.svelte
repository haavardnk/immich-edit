<script lang="ts">
  import { AUTO_ACTIONS, type SectionAutoScope } from '#lib/editor/autoScope.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiAutoFix } from '@mdi/js';

  let { scope, class: className = '' }: { scope: SectionAutoScope; class?: string } = $props();

  const label = $derived(AUTO_ACTIONS[scope]);
  const running = $derived(editor.autoRunning === scope);
  const title = $derived(running ? 'Analyzing…' : label);
</script>

{#if running}
  <span class="px-1 text-[10px] text-dark/65 touch:text-[12px] {className}" role="status">
    Analyzing…
  </span>
{/if}
<IconButton
  size="tiny"
  variant="ghost"
  color={running ? 'primary' : 'secondary'}
  class="size-6 {running ? 'bg-primary/10' : ''} {className}"
  icon={mdiAutoFix}
  {title}
  aria-label={label}
  aria-busy={running}
  disabled={editor.autoBusy || !editor.assetId}
  onclick={() => void editor.onAutoAdjust(scope)}
/>
