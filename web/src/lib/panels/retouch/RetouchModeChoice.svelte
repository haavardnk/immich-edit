<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import {
    RETOUCH_MODE_ICON,
    RETOUCH_MODE_LABEL,
    RETOUCH_MODES
  } from '#lib/editor/retouch/modes.ts';
  import { selectedStroke } from '#lib/editor/retouch/settings.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  const BLURB = {
    heal: { bind: 'retouchHeal', text: 'blends texture from the source into the target' },
    clone: { bind: 'retouchClone', text: 'copies the source pixels exactly' }
  } as const;

  const mode = $derived(selectedStroke()?.stroke.mode ?? editor.retouchTool.mode);
  const options = RETOUCH_MODES.map((value) => ({
    value,
    label: RETOUCH_MODE_LABEL[value],
    icon: RETOUCH_MODE_ICON[value],
    title: `${hint(RETOUCH_MODE_LABEL[value], BLURB[value].bind)} — ${BLURB[value].text}`
  }));
</script>

<SegmentedChoice
  {options}
  value={mode}
  label="Retouch tool"
  fill={layout.mode === 'touch'}
  onChange={editor.setRetouchMode}
/>
