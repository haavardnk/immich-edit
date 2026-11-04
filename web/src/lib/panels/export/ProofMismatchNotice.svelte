<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
  import { spaceLabel } from '#lib/panels/export/settings.ts';
  import { Button } from '@immich/ui';

  const mismatch = $derived(
    editor.isProofing && editor.proofSpace !== exportSettings.form.colorSpace
  );
</script>

{#if mismatch}
  <Notice
    color="warning"
    message={`Soft proofing ${spaceLabel(editor.proofSpace)}, exporting ${spaceLabel(exportSettings.form.colorSpace)}`}
  >
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      class="h-6 panel-action"
      onclick={() => (exportSettings.form.colorSpace = editor.proofSpace)}
    >
      Export {spaceLabel(editor.proofSpace)}
    </Button>
  </Notice>
{/if}
