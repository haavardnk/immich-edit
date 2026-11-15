<script lang="ts">
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import type { Snippet } from 'svelte';
  import ExportPresetMenu from './ExportPresetMenu.svelte';
  import type { PresetExporter } from './presetExporter';

  let {
    open,
    onOpenChange,
    onExited,
    title,
    exporter,
    shareHints = true,
    fitContent = false,
    detent = $bindable(1),
    header,
    footer,
    children
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onExited?: () => void;
    title: string;
    exporter: PresetExporter;
    shareHints?: boolean;
    fitContent?: boolean;
    detent?: number;
    header?: Snippet;
    footer: Snippet;
    children: Snippet;
  } = $props();
</script>

{#snippet sheetHeader()}
  {@render header?.()}
  {#if !fitContent}
    <ExportPresetMenu {exporter} {shareHints} align="end" />
  {/if}
{/snippet}

<BottomSheet
  {open}
  {onOpenChange}
  {onExited}
  {title}
  close="icon"
  closeLabel="Close export"
  detents={fitContent ? ['content'] : [{ fraction: 0.55 }, { fraction: 1 }]}
  bind:detent
  scrim={!fitContent && detent === 0 ? 'light' : 'dim'}
  bodyClass="px-3 py-2"
  header={sheetHeader}
  {footer}
>
  {@render children()}
</BottomSheet>
