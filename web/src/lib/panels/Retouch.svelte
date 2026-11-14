<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import { selectedStroke } from '#lib/editor/retouch/settings.ts';
  import { RETOUCH_LIMIT_TEXT, strokeTitle } from '#lib/editor/retouch/text.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { retouchView } from '#lib/stores/editor/retouchView.svelte.ts';
  import RetouchModeChoice from './retouch/RetouchModeChoice.svelte';
  import RetouchSliders from './retouch/RetouchSliders.svelte';
  import RetouchSourceRow from './retouch/RetouchSourceRow.svelte';
  import RetouchStrokeList from './retouch/RetouchStrokeList.svelte';

  const selected = $derived(selectedStroke());
</script>

<div class="flex flex-col gap-1">
  <div class="flex h-8 items-center justify-between border-b border-hairline">
    <div class="text-[9px] font-semibold uppercase text-dark/65">Tool</div>
    <RetouchModeChoice />
  </div>

  <RetouchSourceRow />
  <CheckboxRow
    label="Visualize spots"
    checked={retouchView.visualize}
    onChange={retouchView.toggleVisualize}
  />

  <div class="flex h-6 items-center text-[10px] font-medium text-dark/65">
    {selected ? strokeTitle(selected.stroke.mode, selected.index) : 'Brush'}
  </div>
  <RetouchSliders />

  <RetouchStrokeList />

  {#if editor.retouchFull}
    <Notice color="warning" message={RETOUCH_LIMIT_TEXT} />
  {/if}
</div>
