<script lang="ts">
  import { isEnabled, runCommand } from '#lib/commands/command.ts';
  import { resetGeometry } from '#lib/commands/editor/edits.ts';
  import { draftAspect, draftPortrait } from '#lib/panels/geometry/ratioActions.ts';
  import { aspectText } from '#lib/panels/geometry/aspects.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui, type GeometryTool } from '#lib/stores/ui.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import {
    mdiAngleAcute,
    mdiCropFree,
    mdiCropLandscape,
    mdiCropPortrait,
    mdiDotsHorizontal,
    mdiFlipHorizontal,
    mdiImageOutline,
    mdiRestore,
    mdiRotateLeft,
    mdiRulerSquare,
    mdiVectorSquare
  } from '@mdi/js';

  let { onratio, onmore }: { onratio: () => void; onmore: () => void } = $props();

  const aspect = $derived(draftAspect());
  const ratioName = $derived(
    aspect.kind === 'ratio' ? aspectText(aspect) : aspect.kind === 'original' ? 'Original' : 'Free'
  );
  const ratioIcon = $derived(
    aspect.kind === 'original'
      ? mdiImageOutline
      : aspect.kind === 'free'
        ? mdiCropFree
        : draftPortrait()
          ? mdiCropPortrait
          : mdiCropLandscape
  );
  const flipH = $derived(editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h);

  const TOOLS: { id: GeometryTool; label: string; name: string; icon: string }[] = [
    { id: 'straighten', label: 'Level', name: 'Level line', icon: mdiAngleAcute },
    { id: 'upright', label: 'Upright', name: 'Guided upright', icon: mdiRulerSquare },
    { id: 'corners', label: 'Corners', name: 'Corners', icon: mdiVectorSquare }
  ];
</script>

{#snippet tool(
  label: string,
  name: string,
  icon: string,
  onclick: () => void,
  pressed: boolean | undefined = undefined,
  disabled = false
)}
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="group h-auto min-w-0 flex-col gap-1 rounded-xl bg-transparent px-0 py-1 text-[11px] font-medium text-dark/80 hover:bg-transparent aria-pressed:text-light-950"
    aria-label={name}
    aria-pressed={pressed}
    {disabled}
    {onclick}
  >
    <span
      class="flex size-11 items-center justify-center rounded-full bg-ghost group-aria-pressed:bg-primary/22 group-aria-pressed:text-primary"
    >
      <Icon {icon} size="22px" aria-hidden="true" />
    </span>
    <span class="max-w-full truncate">{label}</span>
  </Button>
{/snippet}

<div class="@container shrink-0 px-2 py-1.5">
  <div
    role="toolbar"
    aria-label="Crop tools"
    class="grid grid-cols-4 items-start gap-0.5 @sm:grid-cols-8"
  >
    {@render tool(ratioName, `Aspect ratio: ${ratioName}`, ratioIcon, onratio)}
    {@render tool('Rotate', 'Rotate left 90°', mdiRotateLeft, () => editor.rotateStep(270))}
    {@render tool('Flip', 'Flip horizontal', mdiFlipHorizontal, () => editor.flipStep('h'), flipH)}
    {#each TOOLS as entry (entry.id)}
      {@render tool(
        entry.label,
        entry.name,
        entry.icon,
        () => ui.toggleGeometryTool(entry.id),
        ui.geometryTool === entry.id
      )}
    {/each}
    {@render tool(
      'Reset',
      'Reset geometry',
      mdiRestore,
      () => runCommand(resetGeometry),
      undefined,
      !isEnabled(resetGeometry)
    )}
    {@render tool('More', 'More crop actions', mdiDotsHorizontal, onmore)}
  </div>
</div>
