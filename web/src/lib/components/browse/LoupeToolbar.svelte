<script lang="ts">
  import { browseView } from '$lib/stores/browseView.svelte';
  import { compare, type CompareMode } from '$lib/stores/compare.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import MenuItem from '$lib/components/MenuItem.svelte';
  import { hint, keysFor } from '$lib/shortcuts/labels';
  import { trashLabel } from '$lib/browse/trash';
  import { Icon, IconButton } from '@immich/ui';
  import {
    mdiArrowLeft,
    mdiCompare,
    mdiContentDuplicate,
    mdiFullscreen,
    mdiImageOutline,
    mdiTriangleOutline,
    mdiInformationOutline,
    mdiSkipNextOutline,
    mdiKeyboardOutline,
    mdiPencilOutline,
    mdiDotsVertical,
    mdiTrashCanOutline,
    mdiViewGridOutline,
    mdiCircleHalfFull
  } from '@mdi/js';

  type Props = {
    filename: string;
    copyBadge: string | null;
    multi: boolean;
    moreActive: boolean;
    onSelectViewMode: (mode: CompareMode) => void;
    onOpenEditor: () => void;
    onTrash: () => void;
  };

  let { filename, copyBadge, multi, moreActive, onSelectViewMode, onOpenEditor, onTrash }: Props =
    $props();

  let viewModeOpen = $state(false);
  let moreOpen = $state(false);

  function selectViewMode(mode: CompareMode): void {
    viewModeOpen = false;
    onSelectViewMode(mode);
  }
</script>

<nav
  aria-label="Loupe toolbar"
  class="relative z-40 grid h-12 shrink-0 grid-cols-[minmax(0,1fr)_auto] items-center border-b border-hairline bg-editor-chrome px-2"
>
  <div class="flex min-w-0 items-center gap-2 overflow-hidden">
    <IconButton
      class="shrink-0"
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      icon={mdiArrowLeft}
      title={hint('Back', 'loupeClose')}
      aria-label={hint('Back', 'loupeClose')}
      onclick={() => browseView.closeLoupe()}
    />
    <div class="min-w-0 border-l border-hairline pl-2">
      <h2 class="truncate text-xs font-semibold text-white/90">{filename}</h2>
      {#if copyBadge}
        <div
          role="status"
          aria-label="Virtual copy"
          class="flex min-w-0 items-center gap-1 font-mono text-[9px] font-medium text-white/65"
        >
          <Icon icon={mdiContentDuplicate} size="12px" aria-hidden="true" />
          <span class="truncate">{copyBadge}</span>
        </div>
      {/if}
    </div>
  </div>

  <div class="flex min-w-0 items-center justify-end gap-0.5">
    <Popover
      open={viewModeOpen}
      anchor="bottom"
      align="end"
      onOpenChange={(open) => (viewModeOpen = open)}
      contentClass="w-48 py-1"
      tooltip={`View mode: ${compare.mode === 'single' ? 'Single' : compare.mode === 'compare' ? 'Compare' : 'Survey'}`}
    >
      {#snippet trigger(popoverProps)}
        <IconButton
          {...popoverProps}
          size="small"
          variant="ghost"
          color={multi ? 'primary' : 'secondary'}
          icon={compare.mode === 'compare'
            ? mdiCompare
            : compare.mode === 'survey'
              ? mdiViewGridOutline
              : mdiImageOutline}
          title=""
          aria-label="View mode"
          aria-expanded={viewModeOpen}
        />
      {/snippet}
      <div class="flex flex-col">
        <MenuItem
          icon={mdiImageOutline}
          label="Single photo"
          pressed={compare.mode === 'single'}
          onclick={() => selectViewMode('single')}
        />
        <MenuItem
          icon={mdiCompare}
          label="Compare"
          keys={keysFor('enterCompare')}
          pressed={compare.mode === 'compare'}
          onclick={() => selectViewMode('compare')}
        />
        <MenuItem
          icon={mdiViewGridOutline}
          label="Survey"
          keys={keysFor('enterSurvey')}
          pressed={compare.mode === 'survey'}
          onclick={() => selectViewMode('survey')}
        />
      </div>
    </Popover>
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiPencilOutline}
      title={hint('Edit', 'openEditor')}
      aria-label={hint('Edit', 'openEditor')}
      onclick={onOpenEditor}
    />
    <Popover
      open={moreOpen}
      anchor="bottom"
      align="end"
      onOpenChange={(open) => (moreOpen = open)}
      contentClass="w-64 py-1"
      tooltip="More loupe actions"
    >
      {#snippet trigger(popoverProps)}
        <IconButton
          size="small"
          variant="ghost"
          color={moreActive ? 'primary' : 'secondary'}
          icon={mdiDotsVertical}
          title=""
          aria-label="More loupe actions"
          {...popoverProps}
        />
      {/snippet}
      <div class="flex flex-col">
        <MenuItem
          icon={mdiInformationOutline}
          label="Info"
          keys={keysFor('toggleInfo')}
          pressed={browseView.loupeInfoOpen}
          onclick={() => {
            moreOpen = false;
            browseView.toggleLoupeInfo();
          }}
        />
        <MenuItem
          icon={mdiFullscreen}
          label="Fullscreen"
          keys={keysFor('fullscreen')}
          onclick={() => {
            moreOpen = false;
            ui.toggleFullscreen();
          }}
        />
        <MenuItem
          icon={mdiTriangleOutline}
          label="Clipping overlay"
          keys={keysFor('clipWarn')}
          pressed={ui.clipWarn}
          onclick={() => {
            moreOpen = false;
            ui.toggleClipWarn();
          }}
        />
        <MenuItem
          icon={mdiCircleHalfFull}
          label="Grey background"
          pressed={ui.greyCanvas}
          onclick={() => {
            moreOpen = false;
            ui.toggleGreyCanvas();
          }}
        />
        <MenuItem
          icon={mdiSkipNextOutline}
          label="Auto-advance"
          pressed={browseView.loupeAutoAdvance}
          onclick={() => {
            moreOpen = false;
            browseView.setLoupeAutoAdvance(!browseView.loupeAutoAdvance);
          }}
        />
        <div class="my-1 border-t border-dark/10"></div>
        <MenuItem
          icon={mdiTrashCanOutline}
          label={trashLabel(copyBadge !== null)}
          keys={keysFor('trash')}
          disabled={multi}
          onclick={() => {
            moreOpen = false;
            onTrash();
          }}
        />
        <div class="my-1 border-t border-dark/10"></div>
        <MenuItem
          icon={mdiKeyboardOutline}
          label="Keyboard shortcuts"
          keys={keysFor('help')}
          onclick={() => {
            moreOpen = false;
            ui.toggleKeybindsHelp();
          }}
        />
      </div>
    </Popover>
  </div>
</nav>
