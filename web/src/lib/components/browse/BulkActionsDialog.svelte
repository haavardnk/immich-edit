<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import Disclosure from '$lib/components/Disclosure.svelte';
  import BulkApplyPresetPanel from '$lib/panels/BulkApplyPreset.svelte';
  import BulkCopyPastePanel from '$lib/panels/BulkCopyPaste.svelte';
  import BulkExportPanel from '$lib/panels/BulkExport.svelte';
  import { Tabs } from 'bits-ui';

  let { onClose }: { onClose: () => void } = $props();

  type BulkTab = 'edit' | 'export';

  let tab = $state<BulkTab>('edit');
  let presetsOpen = $state(true);
</script>

<Dialog title="Edit and export selected" size="medium" {onClose} bodyClass="min-h-96 p-0!">
  <Tabs.Root
    bind:value={() => tab, (value) => (tab = value as BulkTab)}
    class="flex min-h-96 flex-col"
  >
    <Tabs.List aria-label="Bulk actions" class="flex border-b border-hairline px-4">
      <Tabs.Trigger
        value="edit"
        class="px-4 py-3 text-xs font-medium transition-colors {tab === 'edit'
          ? 'border-b-2 border-primary text-primary'
          : 'text-dark/65 hover:text-dark'}"
      >
        Edit
      </Tabs.Trigger>
      <Tabs.Trigger
        value="export"
        class="px-4 py-3 text-xs font-medium transition-colors {tab === 'export'
          ? 'border-b-2 border-primary text-primary'
          : 'text-dark/65 hover:text-dark'}"
      >
        Export
      </Tabs.Trigger>
    </Tabs.List>
    <Tabs.Content value="edit" class="min-h-0 flex-1 overflow-y-auto py-2">
      {#if tab === 'edit'}
        <BulkCopyPastePanel />
        <Disclosure
          open={presetsOpen}
          title="Presets"
          onOpenChange={(open) => (presetsOpen = open)}
        >
          <BulkApplyPresetPanel />
        </Disclosure>
      {/if}
    </Tabs.Content>
    <Tabs.Content value="export" class="min-h-0 flex-1 overflow-y-auto py-2">
      {#if tab === 'export'}
        <BulkExportPanel />
      {/if}
    </Tabs.Content>
  </Tabs.Root>
</Dialog>
