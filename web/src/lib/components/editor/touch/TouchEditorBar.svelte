<script lang="ts">
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import EditorMoreMenu from '#lib/components/editor/EditorMoreMenu.svelte';
  import { runCommand } from '#lib/commands/command.ts';
  import { redo, undo } from '#lib/commands/editor/edits.ts';
  import { editorBack } from '#lib/commands/editor/navigation.ts';
  import { beforeAfter, holdOriginal } from '#lib/commands/editor/view.ts';
  import { HOLD_ORIGINAL_MS } from '#lib/editor/viewerGestures.ts';
  import { clickGuard } from '#lib/gestures/clickGuard.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { longPressRecognizer } from '#lib/gestures/longPress.ts';
  import ExportPanel from '#lib/panels/Export.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDotsHorizontal, mdiExportVariant } from '@mdi/js';

  let { floating = false, filename = false }: { floating?: boolean; filename?: boolean } = $props();

  let sharing = $state(false);
  const guard = clickGuard();

  const compareHold = gestures({
    recognizers: [
      longPressRecognizer({
        thresholds: { longPressMs: HOLD_ORIGINAL_MS },
        start: () => {
          guard.arm();
          runCommand(holdOriginal);
        },
        end: () => holdOriginal.release(),
        cancel: () => holdOriginal.release()
      })
    ]
  });
</script>

<nav
  aria-label="Editor toolbar"
  class="flex h-12 shrink-0 items-center gap-0.5 px-1.5 {floating
    ? 'absolute inset-x-0 top-0 z-30 bg-editor-chrome/70 backdrop-blur-sm'
    : 'bg-editor-chrome'}"
>
  <CommandIconButton command={editorBack} size="medium" variant="ghost" shape="round" />
  {#if filename && editor.asset}
    <span class="min-w-0 truncate px-1 text-[13px] font-semibold text-light-950/90">
      {editor.asset.originalFileName}
    </span>
  {/if}
  <CommandIconButton command={undo} size="medium" variant="ghost" shape="round" />
  <CommandIconButton command={redo} size="medium" variant="ghost" shape="round" />
  <div class="flex-1"></div>
  <div class="flex" {@attach compareHold} {@attach guard.attachment}>
    <CommandIconButton command={beforeAfter} size="medium" variant="ghost" shape="round" />
  </div>
  <IconButton
    size="medium"
    variant="ghost"
    color="secondary"
    shape="round"
    icon={mdiExportVariant}
    title="Share"
    aria-label="Share"
    disabled={!editor.assetId}
    onclick={() => (sharing = true)}
  />
  <EditorMoreMenu icon={mdiDotsHorizontal} />
</nav>

<BottomSheet
  open={sharing}
  onOpenChange={(open) => (sharing = open)}
  title="Export"
  close="done"
  detents={[{ fraction: 1 }]}
  bodyClass="px-3 py-2"
>
  <ExportPanel />
</BottomSheet>
