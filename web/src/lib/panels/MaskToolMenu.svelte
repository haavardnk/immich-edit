<script lang="ts">
  import MaskToolPicker from '#lib/components/editor/MaskToolPicker.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskModels } from '#lib/stores/maskModels.svelte.ts';
  import { modeVerb, runPick, type MaskTarget } from './maskTools';
  import type { ToolPick } from './maskToolCatalog';
  import type { Snippet } from 'svelte';

  let {
    open,
    align,
    target,
    heading = false,
    onOpenChange,
    trigger
  }: {
    open: boolean;
    align: 'start' | 'end';
    target: MaskTarget;
    heading?: boolean;
    onOpenChange: (open: boolean) => void;
    trigger: Snippet<[Record<string, unknown>]>;
  } = $props();

  function pick(choice: ToolPick): void {
    onOpenChange(false);
    runPick(target, choice);
  }
</script>

<Popover {open} {align} {onOpenChange} {trigger}>
  {#if heading}
    <div
      class="px-3 pt-2 text-[10px] uppercase tracking-wider text-dark/65 border-b border-dark/10 pb-2"
    >
      {modeVerb(target.mode)} with
    </div>
  {/if}
  <MaskToolPicker
    aiKinds={maskModels.kinds}
    semanticClasses={maskModels.semanticClasses}
    aiUnavailable={maskModels.unavailable}
    busy={editor.maskGenerating}
    onpick={pick}
  />
</Popover>
