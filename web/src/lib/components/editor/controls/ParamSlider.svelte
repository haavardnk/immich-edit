<script lang="ts">
  import EditSlider from './EditSlider.svelte';
  import type { ParamDef } from '#lib/editor/params/types.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  let { param }: { param: ParamDef } = $props();
</script>

<EditSlider
  label={param.label}
  commitAction={param.label}
  bind:value={() => param.get(editor.edits), (value) => param.set(editor.edits, value)}
  min={param.min}
  max={param.max}
  step={param.step}
  scale={param.scale}
  defaultValue={param.defaultValue}
  format={param.format}
  gradient={param.gradient}
/>
