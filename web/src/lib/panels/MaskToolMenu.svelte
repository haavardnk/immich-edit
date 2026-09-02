<script lang="ts">
  import MaskToolPicker from '$lib/components/editor/MaskToolPicker.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import type { MaskKind } from '$lib/api/masks';
  import { editor } from '$lib/stores/editor.svelte';
  import { maskModels } from '$lib/stores/maskModels.svelte';
  import {
    addAiTool,
    addBackground,
    addManualTool,
    armBoxTool,
    modeVerb,
    type MaskTarget
  } from './maskTools';
  import type { ManualTool } from '$lib/types/masks';
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

  function pickManual(tool: ManualTool): void {
    onOpenChange(false);
    void addManualTool(target, tool);
  }

  function pickAi(kind: MaskKind, installed: boolean, maskClass?: string): void {
    onOpenChange(false);
    void addAiTool(target, kind, installed, maskClass);
  }

  function pickBox(): void {
    onOpenChange(false);
    armBoxTool(target);
  }

  function pickBackground(): void {
    onOpenChange(false);
    void addBackground(target);
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
    onManual={pickManual}
    onAi={pickAi}
    onBox={pickBox}
    onBackground={pickBackground}
  />
</Popover>
