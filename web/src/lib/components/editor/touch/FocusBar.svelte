<script lang="ts">
  import ChoiceChips from '#lib/components/ChoiceChips.svelte';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import { commitParamReset } from '#lib/components/editor/controls/paramReset.ts';
  import { dragBounds, positionOf } from '#lib/components/editor/controls/relativeDrag.ts';
  import { beforeAfter } from '#lib/commands/editor/view.ts';
  import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
  import { paramDefault } from '#lib/editor/params/define.ts';
  import type { ParamDef } from '#lib/editor/params/types.ts';
  import type { ViewerAdjust } from '#lib/editor/viewerGestures.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { panRecognizer } from '#lib/gestures/pan.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiCheck, mdiRestore } from '@mdi/js';
  import { focusAdjust } from './focusAdjust';

  let { param }: { param: ParamDef } = $props();

  let ruler = $state<HTMLDivElement | null>(null);
  let adjusting: ViewerAdjust | null = null;

  const siblings = $derived(
    TOUCH_GROUPS.flatMap((group) => group.views).find((view) =>
      view.params.some((entry) => entry.id === param.id)
    )?.params ?? [param]
  );
  const value = $derived(param.get(editor.edits, editor.isRaw));
  const bounds = $derived(dragBounds(param));
  const span = $derived(bounds.max - bounds.min || 1);
  const at = $derived((positionOf(value, param) - bounds.min) / span);
  const origin = $derived(
    (positionOf(paramDefault(param, editor.isRaw), param) - bounds.min) / span
  );

  function stop(): void {
    adjusting?.end();
    adjusting = null;
  }

  const rulerDrag = gestures({
    recognizers: [
      panRecognizer({
        lockAxis: true,
        thresholds: { panSlop: 8 },
        start: (pan) => {
          if (pan.axis !== 'x') return;
          adjusting = focusAdjust(param);
          adjusting.start();
        },
        move: (pan) => adjusting?.move(-pan.stepX, (ruler?.clientWidth ?? 0) * 2),
        end: stop,
        cancel: stop
      })
    ]
  });
</script>

<section
  aria-label="Focus {param.label}"
  class="shrink-0 border-t border-hairline bg-editor-panel pb-(--tab-bar-pad)"
>
  <div class="flex items-baseline justify-center gap-2 pt-3 text-[15px] font-medium text-light-950">
    {param.label}
    <span class="font-mono text-sm text-primary tabular-nums" data-testid="focus-value"
      >{param.format(value)}</span
    >
  </div>
  <div
    bind:this={ruler}
    aria-hidden="true"
    data-testid="focus-ruler"
    class="relative h-12 overflow-hidden text-dark/40"
    {@attach rulerDrag}
  >
    <div
      class="absolute inset-y-3 left-1/2 w-[200%] bg-[repeating-linear-gradient(to_right,currentColor_0_1px,transparent_1px_12px)]"
      style:transform="translateX(-{at * 100}%)"
    >
      <span
        class="absolute -top-2.5 h-[calc(100%+0.625rem)] w-px bg-light-950/70"
        style:left="{origin * 100}%"
      ></span>
    </div>
    <span class="absolute inset-y-1 left-1/2 w-0.5 -translate-x-1/2 rounded-full bg-primary"></span>
  </div>
  <div class="px-3">
    <ChoiceChips
      label="Settings"
      options={siblings.map((entry) => ({ value: entry.id, label: entry.label }))}
      value={param.id}
      onSelect={touchUi.focus}
    />
  </div>
  <div class="flex items-center justify-between px-3 py-1.5">
    <IconButton
      size="large"
      variant="ghost"
      color="secondary"
      shape="round"
      icon={mdiRestore}
      title="Reset {param.label}"
      aria-label="Reset {param.label}"
      onclick={() => commitParamReset([param], `Reset ${param.label}`)}
    />
    <CommandIconButton command={beforeAfter} size="large" variant="ghost" shape="round" />
    <IconButton
      size="large"
      variant="filled"
      color="primary"
      shape="round"
      icon={mdiCheck}
      title="Done"
      aria-label="Done"
      onclick={() => touchUi.focus(null)}
    />
  </div>
</section>
