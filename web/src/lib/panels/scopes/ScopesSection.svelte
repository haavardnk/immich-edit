<script lang="ts">
  import { Icon, IconButton } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight, mdiPin, mdiPinOutline } from '@mdi/js';
  import { Collapsible } from 'bits-ui';
  import ScopesPanel from '../Scopes.svelte';
  import ScopeModeSelect from './ScopeModeSelect.svelte';
  import { scopes } from '$lib/stores/scopes.svelte';

  let { open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void } = $props();
</script>

<Collapsible.Root bind:open={() => open, onOpenChange} class="border-t border-hairline">
  <div
    class="flex h-8 items-center gap-1 pr-1 transition-colors {open
      ? 'bg-white/4'
      : 'hover:bg-white/3'}"
  >
    <Collapsible.Trigger
      class="group flex h-8 min-w-0 flex-1 items-center gap-2 px-3 text-[11px] font-semibold transition-colors select-none {open
        ? 'text-dark'
        : 'text-dark/65 hover:text-dark'}"
    >
      <Icon
        icon={open ? mdiChevronDown : mdiChevronRight}
        size="14px"
        class="text-dark/35 transition-colors group-hover:text-dark/60"
        aria-hidden="true"
      />
      Scopes
    </Collapsible.Trigger>
    {#if open}
      <ScopeModeSelect />
    {/if}
    <IconButton
      size="tiny"
      variant="ghost"
      color={scopes.pinned ? 'primary' : 'secondary'}
      icon={scopes.pinned ? mdiPin : mdiPinOutline}
      title={scopes.pinned ? 'Unpin scopes' : 'Pin scopes above the panels'}
      aria-label="Pin scopes"
      aria-pressed={scopes.pinned}
      onclick={scopes.togglePinned}
    />
  </div>
  <Collapsible.Content>
    {#if open}
      <div class="bg-black/10 p-1">
        <ScopesPanel />
      </div>
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
