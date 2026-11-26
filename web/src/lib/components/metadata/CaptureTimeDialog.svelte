<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import CaptureTimeBody from '#lib/components/metadata/CaptureTimeBody.svelte';
  import { CONTENT_CAP, type Detent } from '#lib/components/sheet/sheetSnap.ts';
  import { captureEditor } from '#lib/stores/captureEditor.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button } from '@immich/ui';

  const TWO_COLUMNS = 768;
  const STACKED: readonly Detent[] = [{ fraction: CONTENT_CAP }];

  const editor = captureEditor;

  let width = $state(0);
  const detents = $derived(width > 0 && width < TWO_COLUMNS ? STACKED : undefined);

  const subtitle = $derived(
    [
      countOf(editor.photos.length, 'photo'),
      editor.cameras.length > 1 ? countOf(editor.cameras.length, 'camera') : ''
    ]
      .filter(Boolean)
      .join(' · ')
  );
</script>

{#if editor.request}
  <Dialog
    title="Capture time"
    {subtitle}
    size="giant"
    onClose={editor.close}
    {detents}
    bodyClass="flex flex-col overflow-hidden p-0 touch:overflow-visible touch:px-5 touch:pt-2 touch:pb-4"
  >
    <CaptureTimeBody bind:width />
    {#snippet footer()}
      <div class="flex w-full items-center gap-2">
        <span class="min-w-0 flex-1 truncate text-xs text-dark/65 touch:hidden">
          {editor.summary}
        </span>
        <Button
          variant="ghost"
          color="secondary"
          class="touch:hidden"
          disabled={editor.saving}
          onclick={editor.close}
        >
          Cancel
        </Button>
        <Button
          class="touch:h-12 touch:flex-1 touch:rounded-full"
          loading={editor.saving}
          disabled={editor.changed === 0}
          onclick={editor.apply}
        >
          {editor.changed === 0 ? 'Apply' : `Apply to ${countOf(editor.changed, 'photo')}`}
        </Button>
      </div>
    {/snippet}
  </Dialog>
{/if}
