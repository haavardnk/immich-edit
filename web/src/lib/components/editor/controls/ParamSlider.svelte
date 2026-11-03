<script lang="ts">
  import EditSlider from './EditSlider.svelte';
  import { paramDefault } from '#lib/editor/params/define.ts';
  import type { ParamDef } from '#lib/editor/params/types.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  let { param, disabled = false }: { param: ParamDef; disabled?: boolean } = $props();
</script>

<EditSlider
  label={param.label}
  commitAction={param.history}
  bind:value={
    () => param.get(editor.edits, editor.isRaw),
    (value) => param.set(editor.edits, value, editor.isRaw)
  }
  min={param.min}
  max={param.max}
  step={param.step}
  scale={param.scale}
  defaultValue={paramDefault(param, editor.isRaw)}
  disabled={disabled || (param.inactive?.(editor.edits, editor.isRaw) ?? false)}
  format={param.format}
  gradient={param.liveGradient?.(editor.edits) ?? param.gradient}
  previewMode={param.previewMode}
/>
