<script lang="ts">
  import DualRangeSlider from '#lib/components/editor/controls/DualRangeSlider.svelte';
  import EditSlider from '#lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '#lib/components/editor/controls/segmentedControl.ts';
  import ModelMissingNotice from '#lib/components/ModelMissingNotice.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { depthMap } from '#lib/stores/editor/depthMap.svelte.ts';
  import { maskModels } from '#lib/stores/maskModels.svelte.ts';
  import {
    BOKEH_SHAPES,
    LENS_BLUR_AMOUNT,
    NEUTRAL_LENS_BLUR,
    lensBlurActive,
    resetLensBlur,
    type BokehShape
  } from '#lib/types/edits.ts';
  import { Button } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';

  const SHAPE_LABELS: Record<BokehShape, string> = {
    circle: 'Circle',
    hexagon: 'Hexagon',
    octagon: 'Octagon'
  };

  const FOCUS_DEFAULT: [number, number] = [
    NEUTRAL_LENS_BLUR.focus_min,
    NEUTRAL_LENS_BLUR.focus_max
  ];

  const blur = $derived(editor.edits.lens_blur);
  const hasDepth = $derived(blur.depth_raster_id !== '');
  const active = $derived(lensBlurActive(blur));
  const busy = $derived(depthMap.busy(editor.assetId));
  const failure = $derived(depthMap.error(editor.assetId));
  const blocked = $derived.by(() => {
    if (maskModels.failed) return 'Could not reach the server to check for AI models.';
    if (!maskModels.enabled) return 'AI models are turned off on this server.';
    return null;
  });
  const depthMissing = $derived(blocked === null && !maskModels.installed('depth'));

  $effect(() => {
    void maskModels.load();
  });

  $effect(() => {
    const assetId = editor.assetId;
    if (!assetId || !editor.initialised) return;
    const rasterId = depthMap.take(assetId);
    if (rasterId) void applyDepth(assetId, rasterId);
  });

  async function applyDepth(assetId: string, rasterId: string): Promise<boolean> {
    if (editor.assetId !== assetId || !editor.initialised) return false;
    const current = editor.edits.lens_blur;
    editor.edits.lens_blur = {
      ...current,
      depth_raster_id: rasterId,
      amount: current.amount > 0 ? current.amount : LENS_BLUR_AMOUNT
    };
    editor.onLive();
    await editor.onCommit('Lens Blur Depth');
    return true;
  }

  function generate(): void {
    const assetId = editor.assetId;
    if (!assetId) return;
    void depthMap.generate(assetId, (rasterId) => applyDepth(assetId, rasterId));
  }

  function reset(): void {
    editor.edits.lens_blur = resetLensBlur(editor.edits.lens_blur);
    void editor.onCommit('Reset Lens Blur');
  }

  function setFocus([near, far]: [number, number]): void {
    editor.edits.lens_blur.focus_min = near;
    editor.edits.lens_blur.focus_max = far;
    editor.onLive();
  }

  function setShape(shape: BokehShape): void {
    if (editor.edits.lens_blur.shape === shape) return;
    editor.edits.lens_blur.shape = shape;
    editor.onLive();
    void editor.onCommit('Lens Blur Bokeh');
  }
</script>

<div class="flex flex-col gap-1">
  <SectionHeader
    title="Blur"
    section="lens_blur"
    modified={active}
    onReset={reset}
    resetTitle="Reset Lens Blur"
  />
  <div class="flex h-7 items-center justify-between gap-2">
    <span class="truncate text-[11px] text-dark/65" role="status">
      {#if busy}
        Generating depth…
      {:else if hasDepth}
        Depth map ready
      {:else}
        No depth map
      {/if}
    </span>
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      disabled={busy || blocked !== null || depthMissing}
      onclick={generate}
    >
      {hasDepth ? 'Regenerate depth' : 'Generate depth'}
    </Button>
  </div>
  {#if blocked}
    <Notice color="warning" message={blocked} />
  {:else if depthMissing}
    <ModelMissingNotice
      message="Lens Blur needs a depth model."
      askAdmin="Ask an administrator to install one."
    />
  {/if}
  {#if failure}
    <Notice message={`Depth map failed: ${failure}`}>
      <Button size="tiny" variant="ghost" color="secondary" onclick={generate}>Try again</Button>
    </Notice>
  {/if}
  {#if hasDepth}
    <EditSlider
      label="Amount"
      commitAction="Lens Blur Amount"
      bind:value={editor.edits.lens_blur.amount}
      min={0}
      max={100}
    />
    <DualRangeSlider
      label="Focal Range"
      value={[blur.focus_min, blur.focus_max]}
      defaultValue={FOCUS_DEFAULT}
      disabled={!active}
      thumbLabels={['Near focus limit', 'Far focus limit']}
      onLive={setFocus}
      onCommit={() => void editor.onCommit('Lens Blur Focal Range')}
    />
    <div class="panel-row h-7 items-center {active ? '' : 'pointer-events-none opacity-50'}">
      <span class="truncate text-[11px] font-medium text-dark/65 select-none">Bokeh</span>
      <RadioGroup.Root
        value={blur.shape}
        disabled={!active}
        onValueChange={(v: string) => setShape(v as BokehShape)}
        orientation="horizontal"
        aria-label="Bokeh shape"
        class="{segmentedControlClass} col-span-2"
      >
        {#each BOKEH_SHAPES as shape (shape)}
          <RadioGroup.Item value={shape} class="{segmentedRadioItemClass} flex-1 px-2">
            {SHAPE_LABELS[shape]}
          </RadioGroup.Item>
        {/each}
      </RadioGroup.Root>
    </div>
    <EditSlider
      label="Boost"
      commitAction="Lens Blur Boost"
      bind:value={editor.edits.lens_blur.boost}
      min={0}
      max={100}
      disabled={!active}
    />
  {/if}
</div>
