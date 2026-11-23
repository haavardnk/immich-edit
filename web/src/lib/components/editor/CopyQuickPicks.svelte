<script lang="ts">
  import type { CopyHints } from '#lib/editor/copyRows.ts';
  import { TOOL_LABELS, selectedTools, type CopyTool } from '#lib/editor/copyTools.ts';
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
  import Chip from '#lib/components/chips/Chip.svelte';
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
  <Chip
    {pressed}
    {icon}
    disabled={tools.length === 0}
    onPressedChange={() => copyDialog.pickOnly(tools)}
  >
    {label}
    {#if detail}
      <span class="text-dark/65">· {detail}</span>
    {/if}
  </Chip>
{/snippet}

<div
  role="group"
  aria-label="Quick pick"
  class="scrollbar-hidden flex flex-nowrap items-center gap-1.5 overflow-x-auto border-b border-hairline pb-3 touch:pb-2"
>
  <span class="me-1 shrink-0 text-caption text-dark/65 @max-[480px]:hidden touch:text-label">
    Quick pick
  </span>
  {@render chip(
    'Last touched',
    mdiHistory,
    touched ? [touched] : [],
    touched ? TOOL_LABELS[touched] : undefined
  )}
  {@render chip('Modified only', mdiCheck, modified)}
</div>
