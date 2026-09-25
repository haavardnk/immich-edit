<script lang="ts">
  import { untrack } from 'svelte';
  import { hint } from '$lib/keybinds';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import CropSection from './geometry/CropSection.svelte';
  import {
    neutralPerspective,
    perspectiveIsIdentity,
    type PerspectiveEdits
  } from '$lib/utils/perspective';
  import { Button, IconButton } from '@immich/ui';
  import {
    mdiRotateLeft,
    mdiRotateRight,
    mdiFlipHorizontal,
    mdiFlipVertical,
    mdiVectorSquare,
    mdiAngleAcute
  } from '@mdi/js';

  $effect(() => {
    const assetId = editor.assetId;
    const initialised = editor.initialised;
    if (!assetId || !initialised) return;
    untrack(() => editor.startGeometrySession());
    return () => {
      ui.straightening = false;
      untrack(() => void editor.finishGeometrySession());
    };
  });

  function rotateLeft(): void {
    editor.rotateStep(270);
  }
  function rotateRight(): void {
    editor.rotateStep(90);
  }
  function toggleFlipH(): void {
    editor.flipStep('h');
  }
  function toggleFlipV(): void {
    editor.flipStep('v');
  }

  const transformModified = $derived(
    !!editor.geometrySession &&
      (Math.abs(editor.geometrySession.draftAngle) > 1e-4 ||
        !perspectiveIsIdentity(editor.geometrySession.draftPerspective))
  );

  const perspectiveSliders: Array<{
    key: keyof PerspectiveEdits & ('vertical' | 'horizontal' | 'aspect');
    label: string;
  }> = [
    { key: 'vertical', label: 'Vertical' },
    { key: 'horizontal', label: 'Horizontal' },
    { key: 'aspect', label: 'Aspect' }
  ];

  function onPerspectiveLive(key: (typeof perspectiveSliders)[number]['key'], v: number): void {
    editor.updateGeometryDraftPerspective({ [key]: v });
  }

  function resetTransform(): void {
    editor.updateGeometryDraftAngle(0);
    editor.updateGeometryDraftPerspective(neutralPerspective());
  }

  function noCommit(): void {}

  function done(): void {
    void editor.finishGeometrySession();
    ui.openTab('develop');
  }

  function cancel(): void {
    editor.cancelGeometrySession();
    ui.openTab('develop');
  }
</script>

<div class="flex flex-col divide-y divide-dark/10">
  {#if editor.geometrySession}
    <CropSection sess={editor.geometrySession} />

    <div class="flex flex-col gap-1 py-1.5">
      <SectionHeader title="Transform" modified={transformModified} onReset={resetTransform}>
        {#snippet actions()}
          <IconButton
            size="tiny"
            variant="ghost"
            color={ui.straightening ? 'primary' : 'secondary'}
            icon={mdiAngleAcute}
            title={hint('Straighten', 'straighten')}
            aria-label="Straighten"
            aria-pressed={ui.straightening}
            onclick={ui.toggleStraighten}
          />
          <IconButton
            size="tiny"
            variant="ghost"
            color={ui.perspectiveCorners ? 'primary' : 'secondary'}
            icon={mdiVectorSquare}
            title={hint('Corner handles', 'perspective')}
            aria-label="Corner handles"
            aria-pressed={ui.perspectiveCorners}
            onclick={ui.togglePerspectiveCorners}
          />
        {/snippet}
      </SectionHeader>
      <SliderRow
        label="Angle"
        value={editor.geometrySession.draftAngle}
        min={-45}
        max={45}
        step={0.1}
        onLive={editor.updateGeometryDraftAngle}
        onCommit={noCommit}
        format={(v: number) => `${v.toFixed(1)}°`}
      />
      {#each perspectiveSliders as s (s.key)}
        <SliderRow
          label={s.label}
          value={editor.geometrySession.draftPerspective[s.key]}
          min={-100}
          max={100}
          step={1}
          onLive={(v: number) => onPerspectiveLive(s.key, v)}
          onCommit={noCommit}
          format={(v: number) => v.toFixed(0)}
        />
      {/each}
    </div>

    <div class="grid grid-cols-2 gap-1 border-t border-hairline pt-1.5">
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-7 panel-action"
        leadingIcon={mdiRotateLeft}
        aria-label="Rotate left 90°"
        onclick={rotateLeft}
      >
        90°
      </Button>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-7 panel-action"
        leadingIcon={mdiRotateRight}
        aria-label="Rotate right 90°"
        onclick={rotateRight}
      >
        90°
      </Button>
      <Button
        size="tiny"
        variant={(editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h)
          ? 'filled'
          : 'ghost'}
        color={(editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h)
          ? 'primary'
          : 'secondary'}
        class="h-7 panel-action"
        leadingIcon={mdiFlipHorizontal}
        aria-pressed={editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h}
        onclick={toggleFlipH}
      >
        Flip Horizontal
      </Button>
      <Button
        size="tiny"
        variant={(editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v)
          ? 'filled'
          : 'ghost'}
        color={(editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v)
          ? 'primary'
          : 'secondary'}
        class="h-7 panel-action"
        leadingIcon={mdiFlipVertical}
        aria-pressed={editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v}
        onclick={toggleFlipV}
      >
        Flip Vertical
      </Button>
    </div>

    <div
      class="sticky bottom-0 -mx-3 grid grid-cols-2 gap-1 border-t border-hairline bg-light px-3 py-1.5"
    >
      <Button
        size="small"
        variant="ghost"
        color="secondary"
        title={hint('Cancel', 'editorEscape')}
        onclick={cancel}
      >
        Cancel
      </Button>
      <Button size="small" color="primary" title={hint('Done', 'geometryDone')} onclick={done}>
        Done
      </Button>
    </div>
  {/if}
</div>
