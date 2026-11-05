<script lang="ts">
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import type { Detent } from '#lib/components/sheet/sheetSnap.ts';

  let {
    modal = true,
    dismissable = true,
    refuse = false,
    detents,
    detentLabels,
    onOpenChange
  }: {
    modal?: boolean;
    dismissable?: boolean;
    refuse?: boolean;
    detents?: Detent[];
    detentLabels?: string[];
    onOpenChange: (open: boolean) => void;
  } = $props();

  let open = $state(true);
  let detent = $state(0);

  function change(next: boolean): void {
    if (!refuse) open = next;
    onOpenChange(next);
  }
</script>

<button type="button">Outside</button>
<output data-testid="detent">{detent}</output>
<BottomSheet
  {open}
  onOpenChange={change}
  title="Sheet title"
  close="icon"
  {modal}
  {dismissable}
  {detents}
  {detentLabels}
  bind:detent
>
  <button type="button">Inside</button>
</BottomSheet>
