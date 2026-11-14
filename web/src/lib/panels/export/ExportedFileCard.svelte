<script lang="ts">
  import { canCopyImages, copyImage } from '#lib/panels/export/clipboardImage.ts';
  import { fileMeasure, fileSpec } from '#lib/panels/export/exportedFile.ts';
  import ExportWarnings from '#lib/panels/export/ExportWarnings.svelte';
  import type { ExportedFile } from '#lib/stores/editor/export.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { errorMessage } from '#lib/utils/errors.ts';
  import { makeObjectUrl, revoke } from '#lib/utils/objectUrl.ts';
  import { Button, Icon } from '@immich/ui';
  import {
    mdiCheckCircleOutline,
    mdiCloudUploadOutline,
    mdiContentCopy,
    mdiDownload,
    mdiFileImageOutline
  } from '@mdi/js';

  let {
    file,
    uploading,
    onSave,
    onUpload
  }: {
    file: ExportedFile;
    uploading: boolean;
    onSave: () => void;
    onUpload: () => void;
  } = $props();

  const copyable = canCopyImages();
  const sized = 'h-8 touch:h-11 touch:text-sm';
  const secondary = `${sized} bg-light-200 hover:bg-light-300`;

  let thumbnail = $state<string | null>(null);
  let undecodable = $state(false);

  $effect(() => {
    const url = makeObjectUrl(file.blob);
    thumbnail = url;
    undecodable = false;
    return () => revoke(url);
  });

  function copy(): void {
    copyImage(file.blob, file.options.format).then(
      () => toasts.push('success', 'Image copied'),
      (e: unknown) => toasts.push('error', `Copy failed: ${errorMessage(e)}`)
    );
  }
</script>

<div
  role="group"
  aria-label="Exported file"
  class="flex flex-col gap-2.5 rounded-xl border border-hairline bg-ghost p-2.5 touch:gap-3 touch:p-3"
>
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
  <div class="grid grid-cols-2 gap-1.5 touch:gap-2">
    <Button size="tiny" color="primary" leadingIcon={mdiDownload} class={sized} onclick={onSave}>
      Save
    </Button>
    {#if copyable}
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        leadingIcon={mdiContentCopy}
        class={secondary}
        onclick={copy}
      >
        Copy image
      </Button>
    {/if}
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      leadingIcon={mdiCloudUploadOutline}
      class="{secondary} {copyable ? 'col-span-2' : ''}"
      loading={uploading}
      disabled={uploading}
      onclick={onUpload}
    >
      Upload to Immich
    </Button>
  </div>
</div>
<ExportWarnings warnings={file.warnings} />
