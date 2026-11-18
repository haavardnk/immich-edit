<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import CommandList from '#lib/components/CommandList.svelte';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import { commandLabel } from '#lib/commands/command.ts';
  import { loupeClose } from '#lib/commands/loupe/dismiss.ts';
  import { loupeOpenEditor, loupeSingle } from '#lib/commands/loupe/navigation.ts';
  import { LOUPE_MORE_MENU, LOUPE_VIEW_MODE_MENU } from '#lib/commands/menus.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiContentDuplicate, mdiDotsVertical } from '@mdi/js';

  type Props = {
    filename: string;
    copyBadge: string | null;
    multi: boolean;
    moreActive: boolean;
  };

  let { filename, copyBadge, multi, moreActive }: Props = $props();

  let viewModeOpen = $state(false);
  let moreOpen = $state(false);

  const viewMode = $derived(
    LOUPE_VIEW_MODE_MENU.flat().find((command) => command.pressed?.()) ?? loupeSingle
  );
</script>

<nav
  aria-label="Loupe toolbar"
  class="relative z-40 grid h-12 shrink-0 grid-cols-[minmax(0,1fr)_auto] items-center border-b border-hairline bg-editor-chrome px-2"
>
  <div class="flex min-w-0 items-center gap-2 overflow-hidden">
    <CommandIconButton
      command={loupeClose}
      class="shrink-0"
      size="small"
      variant="ghost"
      shape="round"
    />
    <div class="min-w-0 border-l border-hairline pl-2">
      <h2 class="truncate text-xs font-semibold text-light-950/90">{filename}</h2>
      {#if copyBadge}
        <div
          role="status"
          aria-label="Virtual copy"
          class="flex min-w-0 items-center gap-1 font-mono text-[9px] font-medium text-light-950/65"
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
      title="View mode"
      tooltip={`View mode: ${commandLabel(viewMode)}`}
    >
      {#snippet trigger(popoverProps)}
        <IconButton
          {...popoverProps}
          size="small"
          variant="ghost"
          color={multi ? 'primary' : 'secondary'}
          icon={viewMode.icon}
          title=""
          aria-label="View mode"
          aria-expanded={viewModeOpen}
        />
      {/snippet}
      <div class="flex flex-col">
        <CommandList menu={LOUPE_VIEW_MODE_MENU} onrun={() => (viewModeOpen = false)} />
      </div>
    </Popover>
    <CommandIconButton command={loupeOpenEditor} size="small" variant="ghost" />
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
        <CommandList menu={LOUPE_MORE_MENU} onrun={() => (moreOpen = false)} />
      </div>
    </Popover>
  </div>
</nav>
