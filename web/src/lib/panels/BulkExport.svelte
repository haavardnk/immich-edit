<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import { createImmichExportJob, createZipExportJob, type JobTarget } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiCloudUpload, mdiFolderZip, mdiLoading } from '@mdi/js';
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
    type Destination,
  } from './export/settings';

  let destination = $state<Destination>('download');
  let form = $state(defaultExportForm());
  let busy = $state(false);

  $effect(() => {
    if (destination === 'immich') ensureLibraryLoaded();
  });

  async function submit(): Promise<void> {
    if (busy) return;
    busy = true;
    const verb = destination === 'immich' ? 'export to Immich' : 'zip download';
    await runBulkJob(
      (target: JobTarget) =>
        destination === 'immich'
          ? createImmichExportJob(target, immichOptions(form))
          : createZipExportJob(target, baseOptions(form), form.filenameSuffix),
      {
        success: (count) => `Queued ${verb} of ${count} asset${count === 1 ? '' : 's'}`,
        error: 'Failed to queue export',
      },
    );
    busy = false;
  }

  let label = $derived(formatLabel(form.format));
</script>

<div class="flex flex-col gap-4 px-4 pt-3">
  <div class="text-[11px] text-immich-dark-fg/60 select-none">
    {selection.targetCount} asset{selection.targetCount === 1 ? '' : 's'} selected
  </div>

  <DestinationToggle bind:value={destination} downloadLabel="Download ZIP" />

  <FormatOptions bind:form />

  <SuffixField bind:form />

  {#if destination === 'immich'}
    <div class="flex flex-col gap-3 pt-3 mt-1 border-t border-white/10">
      <ImmichOptions bind:form radioName="bulkStackPrimary" />
    </div>
  {/if}

  <button
    class="flex items-center justify-center gap-2 py-2.5 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={busy || selection.targetCount === 0}
    onclick={() => void submit()}
  >
    {#if busy}
      <Icon path={mdiLoading} size={16} class="animate-spin" />
    {:else}
      <Icon path={destination === 'immich' ? mdiCloudUpload : mdiFolderZip} size={16} />
    {/if}
    {destination === 'immich'
      ? `Export ${selection.targetCount} to Immich`
      : `Download ${selection.targetCount} as ${label} ZIP`}
  </button>

  <p class="text-[11px] leading-relaxed text-immich-dark-fg/40">
    Runs as a background job. Track progress in the Jobs panel.
  </p>

  <div class="h-4"></div>
</div>

