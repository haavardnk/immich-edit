<script lang="ts">
  import ContextMenu from '#lib/components/ContextMenu.svelte';
  import ContextMenuItem from '#lib/components/ContextMenuItem.svelte';
  import Dialog from '#lib/components/Dialog.svelte';
  import MenuItem from '#lib/components/MenuItem.svelte';
  import Popover from '#lib/components/Popover.svelte';

  let {
    kind,
    onClose,
    onOpenChange,
    onRow,
    closeOnRow = false
  }: {
    kind: 'dialog' | 'menu' | 'panel' | 'control' | 'context';
    onClose: () => void;
    onOpenChange: (open: boolean) => void;
    onRow: () => void;
    closeOnRow?: boolean;
  } = $props();

  let open = $state(false);

  function change(next: boolean): void {
    open = next;
    onOpenChange(next);
  }

  function runRow(): void {
    onRow();
    if (closeOnRow) change(false);
  }
</script>

{#if kind === 'dialog'}
  <Dialog title="Dialog title" {onClose}>
    <p>Dialog body</p>
  </Dialog>
{:else if kind === 'context'}
  <ContextMenu>
    {#snippet trigger(props)}
      <button type="button" {...props}>Thumb</button>
    {/snippet}
    <ContextMenuItem onSelect={onRow}>Open row</ContextMenuItem>
  </ContextMenu>
{:else}
  <Popover
    {open}
    onOpenChange={change}
    appearance={kind}
    title="Popover title"
    tooltip="Popover tooltip"
    contentClass="max-h-80"
  >
    {#snippet trigger(props)}
      <button type="button" {...props}>Toggle</button>
    {/snippet}
    <MenuItem icon="M0 0" label="Run row" onclick={runRow} />
  </Popover>
{/if}
