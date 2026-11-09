<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import Viewer from '#lib/components/editor/Viewer.svelte';
  import Filmstrip from '#lib/components/shell/Filmstrip.svelte';
  import { provideSliderFocus } from '#lib/components/editor/controls/sliderFocus.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { reveal } from '#lib/motion/reveal.ts';
  import { PARAMS } from '#lib/editor/params/catalog.ts';
  import type { ViewerAdjust } from '#lib/editor/viewerGestures.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { valueEntry, type ValueEntryTarget } from '#lib/stores/valueEntry.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import FocusBar from './FocusBar.svelte';
  import ModeBar from './ModeBar.svelte';
  import TouchEditorBar from './TouchEditorBar.svelte';
  import TouchPanel from './TouchPanel.svelte';
  import Tray from './Tray.svelte';
  import ValueKeypad from './ValueKeypad.svelte';
  import { focusAdjust } from './focusAdjust';
  import { viewportInset, windowViewport } from '#lib/components/sheet/visualViewport.svelte.ts';

  provideSliderFocus({ focus: (param) => touchUi.focus(param.id) });

  const arrangement = $derived(layout.wide ? 'inspector' : layout.landscape ? 'side' : 'stack');
  const stack = $derived(arrangement === 'stack');
  const focused = $derived(PARAMS.find((param) => param.id === touchUi.focused) ?? null);
  const chrome = $derived(!ui.fullscreen && !focused);
  const width = $derived(arrangement === 'inspector' ? 'w-90' : 'w-75');
  let available = $state(0);
  let region = $state<HTMLElement>();
  let bar = $state(0);
  const keyboard = viewportInset();
  const lift = $derived(touchUi.searching ? bar : 0);
  const docked = $derived(touchUi.searching ? available + keyboard.bottom - bar : available);

  const bottom = $derived(
    focused ? 'focus' : valueEntry.target ? 'entry' : chrome && !touchUi.searching ? 'modes' : null
  );

  $effect(() => {
    if (touchUi.searching) return keyboard.watch(windowViewport());
  });

  $effect(() => {
    void bottom;
    if (region) available = region.clientHeight;
  });

  function adjust(): ViewerAdjust | null {
    const param = focused;
    if (!param || param.inactive?.(editor.edits, editor.isRaw)) return null;
    return focusAdjust(param);
  }

  function tapPhoto(): void {
    if (!focused) ui.toggleFullscreen();
  }
</script>

{#snippet keypad(entry: ValueEntryTarget)}
  {#key entry}
    <ValueKeypad target={entry} />
  {/key}
{/snippet}

<div
  class="app-frame flex flex-col overflow-hidden bg-editor-chrome {stack ? 'pb-0!' : ''}"
  data-arrangement={arrangement}
>
  {#if editor.error}
    <Notice message={editor.error} class="mx-4 my-2 text-xs" />
  {/if}
  {#if chrome && arrangement !== 'side'}
    <TouchEditorBar filename={arrangement === 'inspector'} />
  {/if}
  <div
    class="flex min-h-0 flex-1 {stack ? 'flex-col' : 'flex-row'}"
    bind:this={region}
    bind:clientHeight={available}
  >
    <div class="relative flex min-h-0 min-w-0 flex-1 flex-col">
      <Viewer ontap={tapPhoto} {adjust} />
      {#if editor.showingOriginal}
        <span
          role="status"
          class="pointer-events-none absolute left-1/2 z-20 -translate-x-1/2 rounded-full bg-editor-chrome/80 px-3 py-1 text-xs font-medium text-light-950 {arrangement ===
          'side'
            ? 'top-15'
            : 'top-3'}">Original</span
        >
      {/if}
      {#if chrome && arrangement === 'side'}
        <TouchEditorBar floating />
      {/if}
      {#if focused && !stack}
        <FocusBar param={focused} />
      {/if}
      {#if chrome && arrangement === 'inspector' && touchUi.filmstrip && editor.assetId}
        <Filmstrip />
      {/if}
    </div>
    {#if stack}
      {#if chrome}
        {#if !ui.rightCollapsed}
          <Tray {available} {docked} {lift} />
        {/if}
      {/if}
    {:else if chrome}
      {#if !ui.rightCollapsed}
        <aside
          aria-label="Adjustments"
          transition:reveal={{ axis: 'x' }}
          class="flex min-h-0 shrink-0 overflow-hidden border-l border-hairline bg-editor-panel"
        >
          <div class="flex min-h-0 shrink-0 flex-col {width}">
            <TouchPanel stacked={arrangement === 'inspector'} />
            {#if valueEntry.target}
              {@render keypad(valueEntry.target)}
            {/if}
          </div>
        </aside>
      {/if}
      <ModeBar rail filmstrip={arrangement === 'inspector'} />
    {/if}
  </div>
  {#if stack}
    {#if focused}
      <FocusBar param={focused} />
    {:else if valueEntry.target}
      {@render keypad(valueEntry.target)}
    {:else if chrome && !touchUi.searching}
      <div class="shrink-0" bind:clientHeight={bar}><ModeBar /></div>
    {/if}
  {/if}
  {#if touchUi.searching}
    <div class="shrink-0" style:height="{keyboard.bottom}px"></div>
  {/if}
</div>
