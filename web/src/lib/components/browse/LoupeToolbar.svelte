<script lang="ts">
  import { browseView } from '$lib/stores/browseView.svelte';
  import { compare, type CompareMode } from '$lib/stores/compare.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { hint } from '$lib/keybinds';
  import { Button, Icon, IconButton } from '@immich/ui';
  import {
    mdiArrowLeft,
    mdiCompare,
    mdiContentDuplicate,
    mdiImageOutline,
    mdiTriangleOutline,
    mdiInformationOutline,
    mdiSkipNextOutline,
    mdiKeyboardOutline,
    mdiPencilOutline,
    mdiDotsVertical,
    mdiViewGridOutline
  } from '@mdi/js';

  type Props = {
    filename: string;
    copyBadge: string | null;
    multi: boolean;
    moreActive: boolean;
    onSelectViewMode: (mode: CompareMode) => void;
    onOpenEditor: () => void;
  };

  let { filename, copyBadge, multi, moreActive, onSelectViewMode, onOpenEditor }: Props = $props();

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
      contentClass="w-44 p-2"
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
          title={`View mode: ${compare.mode === 'single' ? 'Single' : compare.mode === 'compare' ? 'Compare' : 'Survey'}`}
          aria-label="View mode"
          aria-expanded={viewModeOpen}
        />
      {/snippet}
      <div class="flex flex-col gap-1">
        <Button
          size="tiny"
          variant="ghost"
          color={compare.mode === 'single' ? 'primary' : 'secondary'}
          class="w-full justify-start"
          leadingIcon={mdiImageOutline}
          aria-pressed={compare.mode === 'single'}
          onclick={() => selectViewMode('single')}>Single photo</Button
        >
        <Button
          size="tiny"
          variant="ghost"
          color={compare.mode === 'compare' ? 'primary' : 'secondary'}
          class="w-full justify-start"
          leadingIcon={mdiCompare}
          aria-pressed={compare.mode === 'compare'}
          onclick={() => selectViewMode('compare')}>{hint('Compare', 'enterCompare')}</Button
        >
        <Button
          size="tiny"
          variant="ghost"
          color={compare.mode === 'survey' ? 'primary' : 'secondary'}
          class="w-full justify-start"
          leadingIcon={mdiViewGridOutline}
          aria-pressed={compare.mode === 'survey'}
          onclick={() => selectViewMode('survey')}>{hint('Survey', 'enterSurvey')}</Button
        >
      </div>
    </Popover>
    <div class="mx-1 hidden h-5 w-px shrink-0 bg-hairline sm:block"></div>
    <IconButton
      size="small"
      variant="ghost"
      color={browseView.loupeInfoOpen ? 'primary' : 'secondary'}
      icon={mdiInformationOutline}
      title={hint('Info', 'toggleInfo')}
      aria-label={hint('Info', 'toggleInfo')}
      aria-pressed={browseView.loupeInfoOpen}
      onclick={() => (browseView.loupeInfoOpen = !browseView.loupeInfoOpen)}
    />
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiPencilOutline}
      title={hint('Edit', 'openEditor')}
      aria-label={hint('Edit', 'openEditor')}
      onclick={onOpenEditor}
    />
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiKeyboardOutline}
      title={hint('Keyboard shortcuts', 'help')}
      aria-label={hint('Keyboard shortcuts', 'help')}
      onclick={() => ui.toggleKeybindsHelp()}
    />
    <Popover
      open={moreOpen}
      anchor="bottom"
      align="end"
      onOpenChange={(open) => (moreOpen = open)}
      contentClass="w-56 p-2"
    >
      {#snippet trigger(popoverProps)}
        <IconButton
          size="small"
          variant="ghost"
          color={moreActive ? 'primary' : 'secondary'}
          icon={mdiDotsVertical}
          title="More loupe actions"
          aria-label="More loupe actions"
          {...popoverProps}
        />
      {/snippet}
      <div class="flex flex-col gap-1">
        <Button
          size="tiny"
          variant="ghost"
          color={browseView.loupeAutoAdvance ? 'primary' : 'secondary'}
          class="w-full justify-start"
          leadingIcon={mdiSkipNextOutline}
          aria-pressed={browseView.loupeAutoAdvance}
          onclick={() => {
            browseView.setLoupeAutoAdvance(!browseView.loupeAutoAdvance);
            moreOpen = false;
          }}>Auto-advance</Button
        >
        <Button
          size="tiny"
          variant="ghost"
          color={ui.clipWarn ? 'primary' : 'secondary'}
          class="w-full justify-start"
          leadingIcon={mdiTriangleOutline}
          aria-pressed={ui.clipWarn}
          onclick={() => {
            ui.toggleClipWarn();
            moreOpen = false;
          }}>Clipping overlay</Button
        >
      </div>
    </Popover>
  </div>
</nav>
