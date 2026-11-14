<script lang="ts">
  import { layout } from '#lib/device/layout.svelte.ts';
  import { TARGET_TOOLS, targetAvailable } from '#lib/editor/targetTools.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { targetAdjust, type TargetTool } from '#lib/stores/targetAdjust.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiCrosshairsGps } from '@mdi/js';

  let { tool, class: className = '' }: { tool: TargetTool; class?: string } = $props();

  const def = $derived(TARGET_TOOLS[tool]);
  const available = $derived(targetAvailable(tool, editor.edits));
  const on = $derived(available && targetAdjust.tool === tool);
  const label = $derived(
    layout.mode === 'touch' ? `Pick ${def.noun} on the photo` : `Adjust ${def.label} on the photo`
  );
  const title = $derived.by(() => {
    if (!available && def.unavailable) return def.unavailable.reason;
    return layout.mode === 'touch' ? label : hint(label, 'targetAdjust');
  });
</script>

<IconButton
  size="tiny"
  variant="ghost"
  color={on ? 'primary' : 'secondary'}
  class="size-6 {on ? 'bg-primary/10' : ''} {className}"
  icon={mdiCrosshairsGps}
  {title}
  aria-label={label}
  aria-pressed={on}
  disabled={!available}
  onclick={() => targetAdjust.toggle(tool)}
/>
