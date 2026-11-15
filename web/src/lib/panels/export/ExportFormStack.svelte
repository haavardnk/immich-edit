<script lang="ts">
  import EarlierExport from '#lib/panels/export/EarlierExport.svelte';
  import ExportForm from '#lib/panels/export/ExportForm.svelte';
  import ExportPresetMenu from '#lib/panels/export/ExportPresetMenu.svelte';
  import ExportResult from '#lib/panels/export/ExportResult.svelte';
  import ProofMismatchNotice from '#lib/panels/export/ProofMismatchNotice.svelte';
  import SaveAutomatically from '#lib/panels/export/SaveAutomatically.svelte';
  import type { EditorExporter } from '#lib/panels/export/editorExport.svelte.ts';
  import type { Snippet } from 'svelte';

  let {
    exporter,
    folded = false,
    presetMenu = true,
    onWatermarkPick,
    children
  }: {
    exporter: EditorExporter;
    folded?: boolean;
    presetMenu?: boolean;
    onWatermarkPick?: () => void;
    children?: Snippet;
  } = $props();
</script>

{#snippet proofNotice()}
  <ProofMismatchNotice />
{/snippet}

{#if presetMenu}
  <ExportPresetMenu {exporter} />
{/if}
<ExportForm
  scope={exporter.scope}
  crop={exporter.crop}
  {folded}
  {onWatermarkPick}
  bind:replaceOriginal={exporter.replaceOriginal}
  fileNotice={proofNotice}
/>
<EarlierExport
  earlier={exporter.earlier}
  query={exporter.earlierQuery}
  replaceReason={exporter.scope.conflictReason}
/>
<ExportResult result={exporter.result} busy={exporter.busy} onRetry={exporter.retry} />
{@render children?.()}
<SaveAutomatically />
