<script lang="ts">
  import { EditorExporter } from '#lib/panels/export/editorExport.svelte.ts';
  import ExportActionBar from '#lib/panels/export/ExportActionBar.svelte';
  import { renderTime } from '#lib/panels/export/exportedFile.ts';
  import ExportedFileActions from '#lib/panels/export/ExportedFileActions.svelte';
  import ExportedFileCard from '#lib/panels/export/ExportedFileCard.svelte';
  import ExportFormStack from '#lib/panels/export/ExportFormStack.svelte';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
  import ExportSheet from '#lib/panels/export/ExportSheet.svelte';
  import { shareSupport } from '#lib/panels/export/shareFile.ts';
  import { outputSummary } from '#lib/panels/export/summary.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiCheckCircle, mdiFileCheckOutline, mdiTuneVariant } from '@mdi/js';

  let { open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void } = $props();

  const exporter = new EditorExporter();

  let detent = $state(1);

  let editing = $derived.by(() => {
    void exporter.file;
    return false;
  });

  const ready = $derived(exporter.file !== null && !editing);
  const output = $derived(outputSummary(exportSettings.form, exporter.crop));
  const status = $derived.by(() => {
    const file = exporter.file;
    if (!file) return '';
    if (file.saved) return 'Saved';
    return shareSupport(file).shareable ? 'Ready to share' : 'Ready to save';
  });
</script>

{#snippet header()}
  {#if ready}
    <Button
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      leadingIcon={mdiTuneVariant}
      class="min-h-11"
      onclick={() => (editing = true)}
    >
      Settings
    </Button>
  {:else if exporter.file}
    <Button
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      leadingIcon={mdiFileCheckOutline}
      class="min-h-11"
      onclick={() => (editing = false)}
    >
      File
    </Button>
  {/if}
{/snippet}

{#snippet summary()}
  {#if output.size}
    <span class="truncate font-mono text-sm text-primary">{output.size}</span>
  {/if}
  <span class="truncate text-xs text-dark/65">{output.spec}</span>
{/snippet}

{#snippet footer()}
  {#if ready && exporter.file}
    <div class="flex w-full flex-col gap-2">
      <ExportedFileActions
        file={exporter.file}
        uploading={exporter.uploading}
        onSave={exporter.save}
        onUpload={exporter.uploadFile}
        compact
      />
    </div>
  {:else}
    <ExportActionBar
      label={exporter.label}
      icon={exporter.icon}
      busy={exporter.busy}
      disabled={exporter.disabled}
      onStart={exporter.start}
      {summary}
    />
  {/if}
{/snippet}

<ExportSheet
  {open}
  {onOpenChange}
  title="Export"
  {exporter}
  fitContent={ready}
  bind:detent
  {header}
  {footer}
>
  {#if ready && exporter.file}
    <div class="flex flex-col gap-3 pb-1">
      <div class="flex items-center gap-2 text-sm">
        <Icon icon={mdiCheckCircle} size="20px" class="text-success" aria-hidden="true" />
        <span class="font-medium text-light-950">{status}</span>
        <span class="ms-auto text-xs text-dark/65">{renderTime(exporter.file.renderMs)}</span>
      </div>
      <ExportedFileCard file={exporter.file} />
    </div>
  {:else}
    <div class="flex flex-col gap-1">
      <ExportFormStack {exporter} folded presetMenu={false} onWatermarkPick={() => (detent = 0)} />
    </div>
  {/if}
</ExportSheet>
