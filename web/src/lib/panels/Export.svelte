<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import { Button, Icon } from '@immich/ui';
  import { mdiExport, mdiCloudUpload, mdiRefresh, mdiAlertOutline } from '@mdi/js';
  import DestinationToggle from './export/DestinationToggle.svelte';
  import FormatOptions from './export/FormatOptions.svelte';
  import ImmichOptions from './export/ImmichOptions.svelte';
  import {
    baseOptions,
    defaultExportForm,
    ensureLibraryLoaded,
    formatLabel,
    immichOptions,
    type Destination
  } from './export/settings';

  let destination = $state<Destination>('download');
  let form = $state(defaultExportForm());

  $effect(() => {
    if (destination === 'immich') ensureLibraryLoaded();
  });

  let isLoading = $derived(
    destination === 'download' ? editor.exporting : editor.exportingToImmich
  );
  let label = $derived(formatLabel(form.format));
  let buttonLabel = $derived(
    destination === 'download' ? `Export ${label}` : `Upload ${label} to Immich`
  );
  let busyLabel = $derived(destination === 'download' ? 'Exporting…' : 'Uploading…');
</script>

<div class="flex flex-col gap-1">
  <DestinationToggle bind:value={destination} />

  <FormatOptions bind:form />

  {#if destination === 'immich'}
    <div class="flex flex-col gap-1 border-t border-hairline pt-1.5">
      <TextInput
        label="Filename suffix"
        compact
        color="neutral"
        class="ring-0 focus-within:ring-1 focus-within:ring-primary"
        bind:value={form.filenameSuffix}
        placeholder="_edit"
      />
      <ImmichOptions bind:form />
    </div>
  {/if}

  {#if destination === 'immich' && editor.lastUpload}
    <Notice
      color={editor.lastUpload.kind === 'success'
        ? 'success'
        : editor.lastUpload.kind === 'duplicate'
          ? 'warning'
          : 'danger'}
      message={editor.lastUpload.message}
    >
      {#if editor.lastUpload.kind === 'error'}
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="h-6 panel-action"
          leadingIcon={mdiRefresh}
          onclick={() => void editor.retryUpload()}
          disabled={isLoading}
        >
          Retry
        </Button>
      {/if}
    </Notice>
  {/if}

  {#if destination === 'immich' && editor.lastWarnings.length > 0}
    <ul
      class="space-y-0.5 rounded-sm border border-warning-500/40 bg-warning-950/40 px-2 py-1.5 text-[10px] leading-snug text-warning-100"
    >
      {#each editor.lastWarnings as w (w)}
        <li class="flex items-start gap-1.5">
          <Icon icon={mdiAlertOutline} size="12px" class="mt-0.5 shrink-0" aria-hidden="true" />
          <span>{w}</span>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="sticky bottom-0 -mx-3 border-t border-hairline bg-light px-3 py-1.5">
    <Button
      size="small"
      color="primary"
      fullWidth
      loading={isLoading}
      leadingIcon={destination === 'download' ? mdiExport : mdiCloudUpload}
      disabled={isLoading || !editor.assetId}
      onclick={() => {
        if (destination === 'download') void editor.onExport(baseOptions(form));
        else void editor.onUploadToImmich(immichOptions(form));
      }}
    >
      {isLoading ? busyLabel : buttonLabel}
    </Button>
  </div>
</div>
