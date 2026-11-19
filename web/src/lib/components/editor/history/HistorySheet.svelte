<script lang="ts">
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { liftToasts } from '#lib/components/sheet/liftToasts.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import HistoryList from './HistoryList.svelte';

  let { open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void } = $props();

  $effect(() => {
    if (!open) return;
    editor.historySteps.sheetOpen = true;
    return () => {
      editor.historySteps.sheetOpen = false;
    };
  });
</script>

<BottomSheet
  {open}
  {onOpenChange}
  title="History"
  subtitle="Newest first · tap a step to jump"
  close="icon"
  closeLabel="Close"
  modal={false}
  detents={[{ fraction: 0.5 }, { fraction: 0.9 }]}
  detentLabels={['Half height', 'Full height']}
  bodyClass="pb-2"
  {@attach liftToasts}
>
  <HistoryList />
</BottomSheet>
