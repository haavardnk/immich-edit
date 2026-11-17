<script lang="ts">
  import type { CopyHints } from '#lib/editor/copyRows.ts';
  import { TOOL_LABELS, selectedTools, type CopyTool } from '#lib/editor/copyTools.ts';
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiCheck, mdiHistory } from '@mdi/js';

  let { hints }: { hints: CopyHints } = $props();

  const checked = $derived(selectedTools(copyDialog.checked));
  const touched = $derived(
    hints.touched && copyDialog.available.includes(hints.touched) ? hints.touched : null
  );
  const modified = $derived(copyDialog.available.filter((tool) => hints.modified.has(tool)));

  function only(tools: readonly CopyTool[]): boolean {
    return tools.length === checked.length && tools.every((tool) => checked.includes(tool));
  }
</script>

{#snippet chip(label: string, icon: string, tools: readonly CopyTool[], detail?: string)}
  {@const pressed = tools.length > 0 && only(tools)}
  <Button
    size="tiny"
    shape="round"
    variant="ghost"
    color={pressed ? 'primary' : 'secondary'}
    class="relative touch:before:absolute touch:before:inset-x-0 touch:before:-inset-y-1 h-7 shrink-0 gap-1 px-2.5 text-caption touch:h-9 touch:px-3 touch:text-label {pressed
      ? 'bg-primary/15'
      : 'bg-ghost'}"
    leadingIcon={icon}
    aria-pressed={pressed}
    disabled={tools.length === 0}
    onclick={() => copyDialog.pickOnly(tools)}
  >
    {label}
    {#if detail}
      <span class="text-dark/65">· {detail}</span>
    {/if}
  </Button>
{/snippet}

<div
  role="group"
  aria-label="Quick pick"
  class="scrollbar-hidden flex flex-nowrap items-center gap-1.5 overflow-x-auto border-b border-hairline pb-3 touch:pb-2"
>
  <span class="me-1 shrink-0 text-caption text-dark/65 @max-[480px]:hidden touch:text-label"
    >Quick pick</span
  >
  {@render chip(
    'Last touched',
    mdiHistory,
    touched ? [touched] : [],
    touched ? TOOL_LABELS[touched] : undefined
  )}
  {@render chip('Modified only', mdiCheck, modified)}
</div>
