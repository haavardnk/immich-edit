<script lang="ts">
  import { loupeClose } from '#lib/commands/loupe/dismiss.ts';
  import { loupeInfo } from '#lib/commands/loupe/view.ts';
  import { LOUPE_PHONE_MORE_MENU, LOUPE_PHONE_MULTI_MORE_MENU } from '#lib/commands/menus.ts';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import LoupeModeSwitch from '#lib/components/browse/LoupeModeSwitch.svelte';
  import LoupeMoreMenu from '#lib/components/browse/LoupeMoreMenu.svelte';
  import LoupeTitle from '#lib/components/browse/LoupeTitle.svelte';
  import { mdiDotsHorizontal } from '@mdi/js';

  let {
    filename,
    copyBadge,
    moreActive,
    multi
  }: { filename: string; copyBadge: string | null; moreActive: boolean; multi: boolean } = $props();

  const button = 'size-11 shrink-0';
</script>

<nav
  aria-label="Loupe toolbar"
  class="absolute inset-x-0 top-0 z-10 grid h-[calc(3.5rem+var(--safe-top))] grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1 border-b border-hairline bg-editor-chrome px-1 pt-(--safe-top)"
>
  <CommandIconButton
    command={loupeClose}
    size="medium"
    variant="ghost"
    shape="round"
    class={button}
  />
  {#if multi}
    <LoupeModeSwitch class="w-full max-w-80 justify-self-center text-light-950/90" />
  {:else}
    <LoupeTitle {filename} {copyBadge} class="text-center text-light-950/90" />
  {/if}
  <div class="flex">
    {#if !multi}
      <CommandIconButton
        command={loupeInfo}
        size="medium"
        variant="ghost"
        shape="round"
        class={button}
      />
    {/if}
    <LoupeMoreMenu
      menu={multi ? LOUPE_PHONE_MULTI_MORE_MENU : LOUPE_PHONE_MORE_MENU}
      active={moreActive}
      icon={mdiDotsHorizontal}
      class={button}
    />
  </div>
</nav>
