<script lang="ts">
  import { fileMeasure, fileSpec } from '#lib/panels/export/exportedFile.ts';
  import type { ExportedFile } from '#lib/stores/editor/export.svelte.ts';
  import { makeObjectUrl, revoke } from '#lib/utils/objectUrl.ts';
  import { Icon } from '@immich/ui';
  import { mdiCheckCircleOutline, mdiFileImageOutline } from '@mdi/js';

  let { file }: { file: ExportedFile } = $props();

  let thumbnail = $state<string | null>(null);
  let undecodable = $state(false);

  $effect(() => {
    const url = makeObjectUrl(file.blob);
    thumbnail = url;
    undecodable = false;
    return () => revoke(url);
  });
</script>

<div class="flex items-center gap-3">
  <div
    class="flex h-13 w-19 shrink-0 items-center justify-center overflow-hidden rounded-md bg-light-200 touch:h-16 touch:w-22"
  >
    {#if thumbnail && !undecodable}
      <img
        src={thumbnail}
        alt=""
        class="size-full object-cover"
        onerror={() => (undecodable = true)}
      />
    {:else}
      <Icon icon={mdiFileImageOutline} size="24px" class="text-dark/45" aria-hidden="true" />
    {/if}
  </div>
  <div class="flex min-w-0 flex-col gap-0.5 text-[11px] leading-snug text-dark/65 touch:text-xs">
    <p class="font-mono text-xs break-all text-dark touch:text-sm">{file.name}</p>
    <p>{fileMeasure(file)}</p>
    <p>{fileSpec(file)}</p>
    <p class="flex items-center gap-1 {file.saved ? 'text-dark/65' : 'text-success'}">
      <Icon icon={mdiCheckCircleOutline} size="12px" aria-hidden="true" />
      {file.saved ? 'Saved' : 'Ready, not saved yet'}
    </p>
  </div>
</div>
