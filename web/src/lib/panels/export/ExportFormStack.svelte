<script lang="ts">
  import ExportForm from '#lib/panels/export/ExportForm.svelte';
  import ExportResult from '#lib/panels/export/ExportResult.svelte';
  import ProofMismatchNotice from '#lib/panels/export/ProofMismatchNotice.svelte';
  import SaveAutomatically from '#lib/panels/export/SaveAutomatically.svelte';
  import type { EditorExporter } from '#lib/panels/export/editorExport.svelte.ts';
  import type { Snippet } from 'svelte';

  let {
    exporter,
    folded = false,
    presetMenu = true,
    destinations,
    children
  }: {
    exporter: EditorExporter;
    folded?: boolean;
    presetMenu?: boolean;
    destinations?: Snippet;
    children?: Snippet;
  } = $props();
</script>

{#snippet proofNotice()}
  <ProofMismatchNotice />
{/snippet}

<ExportForm
  scope={exporter.scope}
  crop={exporter.crop}
  {folded}
  {presetMenu}
  {destinations}
  bind:replaceOriginal={exporter.replaceOriginal}
  fileNotice={proofNotice}
/>
<ExportResult result={exporter.result} busy={exporter.busy} onRetry={exporter.retry} />
{@render children?.()}
<SaveAutomatically />
