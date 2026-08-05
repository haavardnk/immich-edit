<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiExport, mdiCloudUpload, mdiLoading, mdiRefresh, mdiAlertOutline } from '@mdi/js';
  import DestinationToggle from './export/DestinationToggle.svelte';
  import FormatOptions from './export/FormatOptions.svelte';
  import SuffixField from './export/SuffixField.svelte';
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

<div class="flex flex-col gap-4 px-4 pt-2">
  <DestinationToggle bind:value={destination} />

  <FormatOptions bind:form />

  {#if destination === 'immich'}
    <div class="flex flex-col gap-3 pt-3 mt-1 border-t border-white/10">
      <SuffixField bind:form />
      <ImmichOptions bind:form />
    </div>
  {/if}

  <button
    class="flex items-center justify-center gap-2 py-2.5 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={isLoading || !editor.assetId}
    onclick={() => {
      if (destination === 'download') void editor.onExport(baseOptions(form));
      else void editor.onUploadToImmich(immichOptions(form));
    }}
  >
    {#if isLoading}
      <Icon path={mdiLoading} size={16} class="animate-spin" />
    {:else}
      <Icon path={destination === 'download' ? mdiExport : mdiCloudUpload} size={16} />
    {/if}
    {isLoading ? busyLabel : buttonLabel}
  </button>

  {#if destination === 'immich' && editor.lastUpload}
    <div
      class="text-[11px] leading-relaxed px-3 py-2 rounded-md border flex items-start justify-between gap-2"
      class:bg-emerald-950={editor.lastUpload.kind === 'success'}
      class:border-emerald-500={editor.lastUpload.kind === 'success'}
      class:text-emerald-100={editor.lastUpload.kind === 'success'}
      class:bg-amber-950={editor.lastUpload.kind === 'duplicate'}
      class:border-amber-500={editor.lastUpload.kind === 'duplicate'}
      class:text-amber-100={editor.lastUpload.kind === 'duplicate'}
      class:bg-red-950={editor.lastUpload.kind === 'error'}
      class:border-red-500={editor.lastUpload.kind === 'error'}
      class:text-red-100={editor.lastUpload.kind === 'error'}
    >
      <span class="flex-1">{editor.lastUpload.message}</span>
      {#if editor.lastUpload.kind === 'error'}
        <button
          class="flex items-center gap-1 px-2 py-0.5 rounded bg-white/10 hover:bg-white/20 transition-colors"
          onclick={() => void editor.retryUpload()}
          disabled={isLoading}
        >
          <Icon path={mdiRefresh} size={12} />
          Retry
        </button>
      {/if}
    </div>
  {/if}

  {#if destination === 'immich' && editor.lastWarnings.length > 0}
    <ul
      class="text-[11px] leading-relaxed px-3 py-2 rounded-md border border-amber-500/40 bg-amber-950/40 text-amber-100 space-y-1"
    >
      {#each editor.lastWarnings as w (w)}
        <li class="flex items-start gap-1.5">
          <Icon path={mdiAlertOutline} size={12} class="mt-0.5 shrink-0" />
          <span>{w}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>
