<script lang="ts">
  import { hint } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui, type GeometryTool } from '#lib/stores/ui.svelte.ts';
  import type { GeometrySession } from '#lib/stores/editor/geometry.svelte.ts';
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import { PERSPECTIVE, STRAIGHTEN } from '#lib/editor/params/geometry.ts';
  import { perspectiveIsIdentity, type PerspectiveEdits } from '#lib/utils/perspective.ts';
  import UprightCard from '#lib/panels/geometry/UprightCard.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiAngleAcute, mdiRulerSquare, mdiVectorSquare } from '@mdi/js';

  let { sess }: { sess: GeometrySession } = $props();

  const modified = $derived(
    Math.abs(sess.draftAngle) > 1e-4 || !perspectiveIsIdentity(sess.draftPerspective)
  );

  const perspectiveSliders: Array<{
    key: keyof PerspectiveEdits & ('vertical' | 'horizontal' | 'aspect');
    label: string;
  }> = [
    { key: 'vertical', label: 'Vertical' },
    { key: 'horizontal', label: 'Horizontal' },
    { key: 'aspect', label: 'Aspect' }
  ];

  const tools: Array<{ id: GeometryTool; label: string; title: string; icon: string }> = [
    {
      id: 'straighten',
      label: 'Straighten',
      title: hint('Straighten', 'straighten'),
      icon: mdiAngleAcute
    },
    { id: 'upright', label: 'Guided upright', title: 'Guided upright', icon: mdiRulerSquare },
    { id: 'corners', label: 'Corner handles', title: 'Corner handles', icon: mdiVectorSquare }
  ];
</script>

<div class="flex flex-col gap-1 py-1.5">
  <SectionHeader title="Transform" {modified} onReset={() => void editor.resetGeometryTransform()}>
    {#snippet actions()}
      {#each tools as tool (tool.id)}
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
  {#if ui.geometryTool === 'upright'}
    <UprightCard {sess} />
  {/if}
  <SliderRow
    label="Angle"
    value={sess.draftAngle}
    range={STRAIGHTEN}
    onLive={editor.updateGeometryDraftAngle}
    onCommit={() => void editor.commitGeometry('Straighten')}
  />
  {#each perspectiveSliders as s (s.key)}
    <SliderRow
      label={s.label}
      value={sess.draftPerspective[s.key]}
      range={PERSPECTIVE}
      onLive={(v: number) => editor.updateGeometryDraftPerspective({ [s.key]: v })}
      onCommit={() => void editor.commitGeometry('Perspective')}
    />
  {/each}
</div>
