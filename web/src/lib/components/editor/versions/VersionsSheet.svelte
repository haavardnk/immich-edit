<script lang="ts">
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { liftToasts } from '#lib/components/sheet/liftToasts.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { untrack } from 'svelte';
  import VersionsList from './VersionsList.svelte';

  let { open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void } = $props();

  const store = editor.versions;
  const count = $derived(countOf(store.items.length, 'version'));
  const subtitle = $derived(editor.asset ? `${editor.asset.originalFileName} · ${count}` : count);

  $effect(() => {
    if (!open) return;
    return () => untrack(() => store.summaries.clear());
  });

  $effect(() => {
    if (!open) return;
    const ids = store.items.map((version) => version.id);
    untrack(() => store.summaries.load(ids));
  });
</script>

<BottomSheet
  {open}
  {onOpenChange}
  title="Versions and copies"
  {subtitle}
  close="icon"
  closeLabel="Close"
  modal={false}
  detents={[{ fraction: 0.5 }, { fraction: 0.9 }]}
  detentLabels={['Half height', 'Full height']}
  bodyClass="pb-2"
  {@attach liftToasts}
>
  <VersionsList />
</BottomSheet>
