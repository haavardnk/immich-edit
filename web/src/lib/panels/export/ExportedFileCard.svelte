<script lang="ts">
  import ExportedFileDetails from '#lib/panels/export/ExportedFileDetails.svelte';
  import ExportWarnings from '#lib/panels/export/ExportWarnings.svelte';
  import type { ExportedFile } from '#lib/stores/editor/export.svelte.ts';
  import type { Snippet } from 'svelte';

  let { file, actions }: { file: ExportedFile; actions?: Snippet } = $props();
</script>

<div
  role="group"
  aria-label="Exported file"
  class="flex flex-col gap-2.5 rounded-xl border border-hairline bg-ghost p-2.5 touch:gap-3 touch:p-3"
>
  <ExportedFileDetails {file} />
  {@render actions?.()}
</div>
<ExportWarnings warnings={file.warnings} />
