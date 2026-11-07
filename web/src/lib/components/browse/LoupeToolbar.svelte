<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import AutoAdvancePill from '#lib/components/browse/AutoAdvancePill.svelte';
  import CommandList from '#lib/components/CommandList.svelte';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import LoupeMoreMenu from '#lib/components/browse/LoupeMoreMenu.svelte';
  import LoupeTitle from '#lib/components/browse/LoupeTitle.svelte';
  import { commandLabel } from '#lib/commands/command.ts';
  import { loupeClose } from '#lib/commands/loupe/dismiss.ts';
  import { loupeOpenEditor, loupeSingle } from '#lib/commands/loupe/navigation.ts';
  import { loupeInfo } from '#lib/commands/loupe/view.ts';
  import { LOUPE_MORE_MENU, LOUPE_VIEW_MODE_MENU } from '#lib/commands/menus.ts';
  import { IconButton } from '@immich/ui';

  type Props = {
    filename: string;
    copyBadge: string | null;
    multi: boolean;
    moreActive: boolean;
  };

  let { filename, copyBadge, multi, moreActive }: Props = $props();

  let viewModeOpen = $state(false);

  const viewMode = $derived(
    LOUPE_VIEW_MODE_MENU.flat().find((command) => command.pressed?.()) ?? loupeSingle
  );
</script>

<nav
  aria-label="Loupe toolbar"
  class="@container relative z-40 grid h-12 shrink-0 grid-cols-[minmax(0,1fr)_auto] items-center border-b border-hairline bg-editor-chrome px-2"
>
  <div class="flex min-w-0 items-center gap-2 overflow-hidden">
    <CommandIconButton
      command={loupeClose}
      class="shrink-0"
      size="small"
      variant="ghost"
      shape="round"
    />
    <LoupeTitle {filename} {copyBadge} class="border-l border-hairline pl-2 text-light-950/90" />
  </div>

  <div class="flex min-w-0 items-center justify-end gap-0.5">
    <AutoAdvancePill class="me-1.5" />
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
    <CommandIconButton command={loupeInfo} size="small" variant="ghost" />
    <LoupeMoreMenu menu={LOUPE_MORE_MENU} active={moreActive} />
  </div>
</nav>
