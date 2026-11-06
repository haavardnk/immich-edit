<script lang="ts">
  import { loupeClose } from '#lib/commands/loupe/dismiss.ts';
  import { loupeInfo } from '#lib/commands/loupe/view.ts';
  import { LOUPE_PHONE_MORE_MENU } from '#lib/commands/menus.ts';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import LoupeMoreMenu from '#lib/components/browse/LoupeMoreMenu.svelte';
  import LoupeTitle from '#lib/components/browse/LoupeTitle.svelte';
  import { mdiDotsHorizontal } from '@mdi/js';

  let {
    filename,
    copyBadge,
    moreActive
  }: { filename: string; copyBadge: string | null; moreActive: boolean } = $props();

  const button = 'pointer-events-auto size-11 shrink-0 text-white';
</script>

<nav
  aria-label="Loupe toolbar"
  class="pointer-events-none absolute inset-x-0 top-0 z-10 grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1 bg-linear-to-b from-black/60 to-transparent px-1 pt-[calc(0.25rem+var(--safe-top))] pb-8 text-white"
>
  <CommandIconButton
    command={loupeClose}
    size="medium"
    variant="ghost"
    shape="round"
    class={button}
  />
  <LoupeTitle {filename} {copyBadge} class="pointer-events-auto text-center" />
  <div class="flex">
    <CommandIconButton
      command={loupeInfo}
      size="medium"
      variant="ghost"
      shape="round"
      class={button}
    />
    <LoupeMoreMenu
      menu={LOUPE_PHONE_MORE_MENU}
      active={moreActive}
      icon={mdiDotsHorizontal}
      class={button}
    />
  </div>
</nav>
