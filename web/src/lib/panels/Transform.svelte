<script lang="ts">
  import { untrack } from 'svelte';
  import { hint } from '$lib/shortcuts/labels';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import CropSection from './geometry/CropSection.svelte';
  import type { Edits, GeometryEdits } from '$lib/types/edits';
  import {
    neutralPerspective,
    perspectiveIsIdentity,
    type PerspectiveEdits
  } from '$lib/utils/perspective';
  import { IconButton } from '@immich/ui';
  import { mdiVectorSquare, mdiAngleAcute } from '@mdi/js';

  $effect(() => {
    const assetId = editor.assetId;
    const initialised = editor.initialised;
    if (!assetId || !initialised) return;
    untrack(() => editor.startGeometrySession());
    return () => {
      ui.straightening = false;
      untrack(() => void editor.endGeometrySession());
    };
  });

  $effect(() => {
    const geometry = $state.snapshot(editor.edits.geometry) as GeometryEdits;
    untrack(() => editor.syncGeometrySession(geometry));
  });

  $effect(() => {
    const edits = $state.snapshot(editor.edits) as Edits;
    untrack(() => editor.refreshGeometryFrame(edits));
  });

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
    void editor.commitGeometry('Reset transform');
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
            title="Corner handles"
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
        onCommit={() => void editor.commitGeometry('Straighten')}
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
          onCommit={() => void editor.commitGeometry('Perspective')}
          format={(v: number) => v.toFixed(0)}
        />
      {/each}
    </div>
  {/if}
</div>
