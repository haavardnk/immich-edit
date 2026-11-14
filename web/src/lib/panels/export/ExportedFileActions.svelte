<script lang="ts">
  import { canCopyImages, copyImage } from '#lib/panels/export/clipboardImage.ts';
  import { shareFile, shareSupport } from '#lib/panels/export/shareFile.ts';
  import type { ExportedFile } from '#lib/stores/editor/export.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { errorMessage } from '#lib/utils/errors.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiCloudUploadOutline, mdiContentCopy, mdiDownload, mdiExportVariant } from '@mdi/js';

  let {
    file,
    uploading,
    onSave,
    onUpload,
    compact = false
  }: {
    file: ExportedFile;
    uploading: boolean;
    onSave: () => void;
    onUpload: () => void;
    compact?: boolean;
  } = $props();

  const copyable = canCopyImages();
  const support = $derived(shareSupport(file));
  const shareFirst = $derived(compact && support.shareable);
  const oddRow = $derived(copyable !== support.shareable);
  const sized = 'h-8 touch:h-11 touch:text-sm';
  const secondary = `${sized} bg-light-200 hover:bg-light-300`;
  const round = 'size-11 shrink-0 bg-light-200 hover:bg-light-300';

  function share(): void {
    shareFile(file).catch((e: unknown) => toasts.push('error', `Share failed: ${errorMessage(e)}`));
  }

  function copy(): void {
    copyImage(file.blob, file.options.format).then(
      () => toasts.push('success', 'Image copied'),
      (e: unknown) => toasts.push('error', `Copy failed: ${errorMessage(e)}`)
    );
  }
</script>

{#snippet saveButton(primary: boolean)}
  <Button
    size="tiny"
    variant={primary ? 'filled' : 'ghost'}
    color={primary ? 'primary' : 'secondary'}
    leadingIcon={mdiDownload}
    class="{primary ? sized : secondary} {compact ? 'min-w-0 flex-1' : ''}"
    onclick={onSave}
  >
    Save
  </Button>
{/snippet}

{#snippet shareButton(primary: boolean)}
  <Button
    size="tiny"
    variant={primary ? 'filled' : 'ghost'}
    color={primary ? 'primary' : 'secondary'}
    leadingIcon={mdiExportVariant}
    class="{primary ? sized : secondary} {compact ? 'min-w-0 flex-1' : ''}"
    onclick={share}
  >
    Share
  </Button>
{/snippet}

{#if compact}
  <div class="ms-auto flex w-full max-w-md items-center gap-2">
    {#if shareFirst}
      {@render shareButton(true)}
      {@render saveButton(false)}
    {:else}
      {@render saveButton(true)}
    {/if}
    {#if copyable}
      <IconButton
        size="medium"
        variant="ghost"
        color="secondary"
        shape="round"
        icon={mdiContentCopy}
        class={round}
        title="Copy image"
        aria-label="Copy image"
        onclick={copy}
      />
    {/if}
    <IconButton
      size="medium"
      variant="ghost"
      color="secondary"
      shape="round"
      icon={mdiCloudUploadOutline}
      class={round}
      title="Upload to Immich"
      aria-label="Upload to Immich"
      loading={uploading}
      disabled={uploading}
      onclick={onUpload}
    />
  </div>
{:else}
  <div class="grid grid-cols-2 gap-1.5">
    {@render saveButton(true)}
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
    {#if support.shareable}
      {@render shareButton(false)}
    {/if}
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      leadingIcon={mdiCloudUploadOutline}
      class="{secondary} {oddRow ? 'col-span-2' : ''}"
      loading={uploading}
      disabled={uploading}
      onclick={onUpload}
    >
      Upload to Immich
    </Button>
  </div>
{/if}
{#if support.reason}
  <p class="text-[11px] leading-snug text-dark/65 touch:text-xs">{support.reason}</p>
{/if}
