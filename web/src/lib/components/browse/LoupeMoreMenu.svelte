<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import CommandList from '#lib/components/CommandList.svelte';
  import type { Menu } from '#lib/commands/menus.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDotsVertical } from '@mdi/js';

  let {
    menu,
    active,
    icon = mdiDotsVertical,
    class: className = ''
  }: { menu: Menu; active: boolean; icon?: string; class?: string } = $props();

  let open = $state(false);
</script>

<Popover
  {open}
  anchor="bottom"
  align="end"
  onOpenChange={(next) => (open = next)}
  contentClass="w-64 py-1"
  tooltip="More loupe actions"
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="small"
      variant="ghost"
      color={active ? 'primary' : 'secondary'}
      class={className}
      {icon}
      title=""
      aria-label="More loupe actions"
      {...popoverProps}
    />
  {/snippet}
  <div class="flex flex-col">
    <CommandList {menu} onrun={() => (open = false)} />
  </div>
</Popover>
