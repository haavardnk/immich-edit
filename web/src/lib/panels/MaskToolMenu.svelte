<script lang="ts">
  import MaskToolPicker from '$lib/components/editor/MaskToolPicker.svelte';
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

  let {
    open,
    anchor,
    align,
    target,
    heading = false,
    onClose
  }: {
    open: boolean;
    anchor: HTMLElement | undefined;
    align: 'left' | 'right';
    target: MaskTarget;
    heading?: boolean;
    onClose: () => void;
  } = $props();

  let menu = $state<HTMLDivElement | undefined>(undefined);
  let pos = $state<{ top: number; left: number; right: number } | null>(null);

  $effect(() => {
    if (!open || !anchor) {
      pos = null;
      return;
    }
    const r = anchor.getBoundingClientRect();
    pos = { top: r.bottom + 4, left: r.left, right: window.innerWidth - r.right };
    function onDown(e: PointerEvent): void {
      const t = e.target as Node;
      if (anchor?.contains(t) || menu?.contains(t)) return;
      onClose();
    }
    window.addEventListener('pointerdown', onDown, true);
    window.addEventListener('resize', onClose);
    window.addEventListener('scroll', onClose, true);
    return () => {
      window.removeEventListener('pointerdown', onDown, true);
      window.removeEventListener('resize', onClose);
      window.removeEventListener('scroll', onClose, true);
    };
  });

  function pickManual(tool: ManualTool): void {
    onClose();
    void addManualTool(target, tool);
  }

  function pickAi(kind: MaskKind, installed: boolean, maskClass?: string): void {
    onClose();
    void addAiTool(target, kind, installed, maskClass);
  }

  function pickBox(): void {
    onClose();
    armBoxTool(target);
  }

  function pickBackground(): void {
    onClose();
    void addBackground(target);
  }
</script>

{#if open && pos}
  <div
    bind:this={menu}
    class="fixed z-50 bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl"
    style={align === 'right'
      ? `top: ${pos.top}px; right: ${pos.right}px`
      : `top: ${pos.top}px; left: ${pos.left}px`}
  >
    {#if heading}
      <div
        class="px-3 pt-2 text-[10px] uppercase tracking-wider text-immich-dark-fg/40 border-b border-white/10 pb-2"
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
  </div>
{/if}
