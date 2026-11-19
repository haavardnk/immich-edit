<script lang="ts">
  import { loupeClose } from '#lib/commands/loupe/dismiss.ts';
  import { loupeInfo } from '#lib/commands/loupe/view.ts';
  import { LOUPE_WIDE_MORE_MENU } from '#lib/commands/menus.ts';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import LoupeEditPill from '#lib/components/browse/LoupeEditPill.svelte';
  import LoupeModeSwitch from '#lib/components/browse/LoupeModeSwitch.svelte';
  import LoupeMoreMenu from '#lib/components/browse/LoupeMoreMenu.svelte';
  import LoupeTitle from '#lib/components/browse/LoupeTitle.svelte';
  import { mdiDotsHorizontal } from '@mdi/js';

  let {
    filename,
    copyBadge,
    moreActive
  }: { filename: string; copyBadge: string | null; moreActive: boolean } = $props();
</script>

<nav
  aria-label="Loupe toolbar"
  class="grid h-14 shrink-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2 border-b border-hairline bg-editor-chrome px-2"
>
  <div class="flex min-w-0 items-center gap-1">
    <CommandIconButton
      command={loupeClose}
      size="medium"
      variant="ghost"
      shape="round"
      class="size-11 shrink-0"
    />
    <LoupeTitle {filename} {copyBadge} class="text-light-950/90" />
  </div>
  <LoupeModeSwitch class="w-80 text-light-950/90" />
  <div class="flex items-center justify-end gap-1">
    <CommandIconButton
      command={loupeInfo}
      size="medium"
      variant="ghost"
      shape="round"
      class="size-11"
    />
    <LoupeEditPill />
    <LoupeMoreMenu
      menu={LOUPE_WIDE_MORE_MENU}
      active={moreActive}
      icon={mdiDotsHorizontal}
      class="size-11"
    />
  </div>
</nav>
