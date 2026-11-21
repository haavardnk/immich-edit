<script lang="ts" generics="A extends string">
  import type { MenuActionInfo, MenuGroups } from '#lib/browse/menuActions.ts';
  import ContextMenuItem from '#lib/components/ContextMenuItem.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';

  let {
    groups,
    actions,
    variant,
    onrun
  }: {
    groups: MenuGroups<A>;
    actions: Readonly<Record<A, MenuActionInfo>>;
    variant: 'context' | 'popover';
    onrun: (action: A) => void;
  } = $props();
</script>

{#each groups as group, index (group.join())}
  {#if index > 0}
    <div class="my-1 h-px bg-hairline touch:hidden" role="separator"></div>
  {/if}
  {#each group as action (action)}
    {@const info = actions[action]}
    {#if variant === 'context'}
      <ContextMenuItem icon={info.icon} danger={info.danger} onSelect={() => onrun(action)}>
        {info.label}
      </ContextMenuItem>
    {:else}
      <MenuRow
        icon={info.icon}
        label={info.label}
        danger={info.danger}
        onclick={() => onrun(action)}
      />
    {/if}
  {/each}
{/each}
