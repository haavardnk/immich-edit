<script lang="ts">
  import { untrack } from 'svelte';
  import { hint } from '$lib/shortcuts/labels';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui, type GeometryTool } from '$lib/stores/ui.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import CropSection from './geometry/CropSection.svelte';
  import type { Edits, GeometryEdits } from '$lib/types/edits';
  import { perspectiveIsIdentity, type PerspectiveEdits } from '$lib/utils/perspective';
  import { IconButton } from '@immich/ui';
  import { mdiVectorSquare, mdiAngleAcute, mdiRulerSquare } from '@mdi/js';

  $effect(() => {
    const assetId = editor.assetId;
    const initialised = editor.initialised;
    if (!assetId || !initialised) return;
    untrack(() => editor.startGeometrySession());
    return () => {
      ui.geometryTool = null;
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

  const geometryTools: Array<{ id: GeometryTool; label: string; title: string; icon: string }> = [
    {
      id: 'straighten',
      label: 'Straighten',
      title: hint('Straighten', 'straighten'),
      icon: mdiAngleAcute
    },
    { id: 'upright', label: 'Guided upright', title: 'Guided upright', icon: mdiRulerSquare },
    { id: 'corners', label: 'Corner handles', title: 'Corner handles', icon: mdiVectorSquare }
  ];

  function onPerspectiveLive(key: (typeof perspectiveSliders)[number]['key'], v: number): void {
    editor.updateGeometryDraftPerspective({ [key]: v });
  }
</script>

<div class="flex flex-col divide-y divide-dark/10">
  {#if editor.geometrySession}
    <CropSection sess={editor.geometrySession} />

    <div class="flex flex-col gap-1 py-1.5">
      <SectionHeader
        title="Transform"
        modified={transformModified}
        onReset={() => void editor.resetGeometryTransform()}
      >
        {#snippet actions()}
          {#each geometryTools as tool (tool.id)}
            <IconButton
              size="tiny"
              variant="ghost"
              color={ui.geometryTool === tool.id ? 'primary' : 'secondary'}
              icon={tool.icon}
              title={tool.title}
              aria-label={tool.label}
              aria-pressed={ui.geometryTool === tool.id}
              onclick={() => ui.toggleGeometryTool(tool.id)}
            />
          {/each}
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
