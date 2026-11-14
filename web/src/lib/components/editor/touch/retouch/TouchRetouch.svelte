<script lang="ts">
  import { RETOUCH_MODE_ICON } from '#lib/editor/retouch/modes.ts';
  import { selectedStroke } from '#lib/editor/retouch/settings.ts';
  import {
    modeName,
    RETOUCH_LIMIT_HINT,
    RETOUCH_LIMIT_TEXT,
    strokeName
  } from '#lib/editor/retouch/text.ts';
  import RetouchModeChoice from '#lib/panels/retouch/RetouchModeChoice.svelte';
  import RetouchSliders from '#lib/panels/retouch/RetouchSliders.svelte';
  import RetouchSourceRow from '#lib/panels/retouch/RetouchSourceRow.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiAlertOutline, mdiDotsHorizontal } from '@mdi/js';
  import { onMount } from 'svelte';
  import RetouchSheet from './RetouchSheet.svelte';
  import RetouchStrokeChips from './RetouchStrokeChips.svelte';

  let { stacked = false }: { stacked?: boolean } = $props();

  let menu = $state(false);

  const selected = $derived(selectedStroke());

  onMount(() => {
    const fresh = editor.edits.retouch.length === 0;
    if (fresh && !editor.retouchAnchor && !editor.retouchSampling) editor.toggleRetouchSampling();
  });
</script>

<section class="flex min-h-0 flex-1 flex-col" data-testid="touch-retouch">
  {#if selected}
    {@const title = strokeName(selected.index)}
    <header class="flex h-14 shrink-0 items-center gap-3 border-b border-hairline ps-4 pe-3">
      <Icon
        icon={RETOUCH_MODE_ICON[selected.stroke.mode]}
        size="20px"
        class="shrink-0 text-primary"
        aria-hidden="true"
      />
      <div class="flex min-w-0 flex-1 flex-col">
        <span class="truncate text-[15px] font-medium text-dark">{title}</span>
        <span class="truncate text-[12px] text-dark/65">
          {modeName(selected.stroke.mode)} · drag the ring to move it
        </span>
      </div>
      <IconButton
        size="medium"
        variant="ghost"
        color="secondary"
        class="size-10 shrink-0"
        icon={mdiDotsHorizontal}
        aria-label="Retouch actions"
        onclick={() => (menu = true)}
      />
      <Button
        type="button"
        size="medium"
        shape="round"
        color="primary"
        class="h-10 shrink-0 px-5"
        onclick={() => (editor.activeRetouchId = null)}
      >
        Done
      </Button>
    </header>
  {/if}
  <div
    class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto overscroll-contain px-4 py-2 scrollbar-hidden"
  >
    <RetouchStrokeChips onmenu={selected ? undefined : () => (menu = true)} />
    {#if editor.retouchFull}
      <div
        role="status"
        data-testid="retouch-limit"
        class="flex shrink-0 items-start gap-2.5 rounded-xl border border-warning-500/25 bg-warning-500/10 px-3 py-2.5"
      >
        <Icon
          icon={mdiAlertOutline}
          size="18px"
          class="mt-px shrink-0 text-warning-800"
          aria-hidden="true"
        />
        <div class="flex min-w-0 flex-col gap-0.5">
          <span class="text-[13.5px] font-medium text-warning-800">{RETOUCH_LIMIT_TEXT}</span>
          <span class="text-[12.5px] text-dark/75">{RETOUCH_LIMIT_HINT}</span>
        </div>
      </div>
    {/if}
    <RetouchModeChoice />
    <RetouchSourceRow />
    <RetouchSliders size={stacked} />
  </div>
</section>

<RetouchSheet open={menu} onclose={() => (menu = false)} />
