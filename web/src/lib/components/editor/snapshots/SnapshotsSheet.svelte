<script lang="ts">
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { liftToasts } from '#lib/components/sheet/liftToasts.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import SnapshotList from './SnapshotList.svelte';

  let { open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void } = $props();

  const count = $derived(countOf(editor.snapshots.items.length, 'snapshot'));
  const subtitle = $derived(editor.asset ? `${editor.asset.originalFileName} · ${count}` : count);
</script>

<BottomSheet
  {open}
  {onOpenChange}
  title="Snapshots"
  {subtitle}
  close="icon"
  closeLabel="Close"
  modal={false}
  detents={[{ fraction: 0.5 }, { fraction: 0.9 }]}
  detentLabels={['Half height', 'Full height']}
  bodyClass="pb-2"
  {@attach liftToasts}
>
  <SnapshotList oncompare={() => onOpenChange(false)} />
</BottomSheet>
