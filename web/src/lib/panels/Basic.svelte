<script lang="ts">
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import { editor } from '$lib/stores/editor.svelte';

  const wbModified = $derived(editor.edits.basic.wb_temp !== 0 || editor.edits.basic.wb_tint !== 0);
  const toneModified = $derived(
    editor.edits.basic.exposure_ev !== 0 ||
      editor.edits.basic.brightness !== 0 ||
      editor.edits.basic.contrast !== 0 ||
      editor.edits.tone.highlights !== 0 ||
      editor.edits.tone.shadows !== 0 ||
      editor.edits.tone.whites !== 0 ||
      editor.edits.tone.blacks !== 0
  );
  const presenceModified = $derived(
    editor.edits.basic.texture !== 0 ||
      editor.edits.basic.clarity !== 0 ||
      editor.edits.basic.dehaze !== 0 ||
      editor.edits.basic.vibrance !== 0 ||
      editor.edits.basic.saturation !== 0
  );

  function resetWb(): void {
    editor.edits.basic.wb_temp = 0;
    editor.edits.basic.wb_tint = 0;
    void editor.onCommit('Reset White Balance');
  }

  function resetTone(): void {
    editor.edits.basic.exposure_ev = 0;
    editor.edits.basic.brightness = 0;
    editor.edits.basic.contrast = 0;
    editor.edits.tone.highlights = 0;
    editor.edits.tone.shadows = 0;
    editor.edits.tone.whites = 0;
    editor.edits.tone.blacks = 0;
    void editor.onCommit('Reset Tone');
  }

  function resetPresence(): void {
    editor.edits.basic.texture = 0;
    editor.edits.basic.clarity = 0;
    editor.edits.basic.dehaze = 0;
    editor.edits.basic.vibrance = 0;
    editor.edits.basic.saturation = 0;
    void editor.onCommit('Reset Presence');
  }
</script>

<div class="flex flex-col divide-y divide-dark/10">
  <div class="flex flex-col gap-1 pb-1.5">
    <SectionHeader title="White Balance" modified={wbModified} onReset={resetWb} />
    <EditSlider
      label="Temperature"
      commitAction="Temperature"
      bind:value={editor.edits.basic.wb_temp}
      min={-100}
      max={100}
      gradient="var(--gradient-temperature)"
    />
    <EditSlider
      label="Tint"
      commitAction="Tint"
      bind:value={editor.edits.basic.wb_tint}
      min={-100}
      max={100}
      gradient="var(--gradient-tint)"
    />
  </div>
  <div class="flex flex-col gap-1 py-1.5">
    <SectionHeader title="Tone" modified={toneModified} onReset={resetTone} />
    <EditSlider
      label="Exposure"
      commitAction="Exposure"
      bind:value={editor.edits.basic.exposure_ev}
      min={-5}
      max={5}
      step={0.05}
      format={(v: number) => v.toFixed(2)}
    />
    <EditSlider
      label="Brightness"
      commitAction="Brightness"
      bind:value={editor.edits.basic.brightness}
      min={-100}
      max={100}
    />
    <EditSlider
      label="Contrast"
      commitAction="Contrast"
      bind:value={editor.edits.basic.contrast}
      min={-100}
      max={100}
    />
    <div class="border-t border-dark/10"></div>
    <EditSlider
      label="Highlights"
      commitAction="Highlights"
      bind:value={editor.edits.tone.highlights}
      min={-100}
      max={100}
    />
    <EditSlider
      label="Shadows"
      commitAction="Shadows"
      bind:value={editor.edits.tone.shadows}
      min={-100}
      max={100}
    />
    <EditSlider
      label="Whites"
      commitAction="Whites"
      bind:value={editor.edits.tone.whites}
      min={-100}
      max={100}
    />
    <EditSlider
      label="Blacks"
      commitAction="Blacks"
      bind:value={editor.edits.tone.blacks}
      min={-100}
      max={100}
    />
  </div>
  <div class="flex flex-col gap-1 pt-1.5">
    <SectionHeader title="Presence" modified={presenceModified} onReset={resetPresence} />
    <EditSlider
      label="Texture"
      commitAction="Texture"
      bind:value={editor.edits.basic.texture}
      min={-100}
      max={100}
    />
    <EditSlider
      label="Clarity"
      commitAction="Clarity"
      bind:value={editor.edits.basic.clarity}
      min={-100}
      max={100}
    />
    <EditSlider
      label="Dehaze"
      commitAction="Dehaze"
      bind:value={editor.edits.basic.dehaze}
      min={-100}
      max={100}
    />
    <div class="border-t border-dark/10"></div>
    <EditSlider
      label="Vibrance"
      commitAction="Vibrance"
      bind:value={editor.edits.basic.vibrance}
      min={-100}
      max={100}
    />
    <EditSlider
      label="Saturation"
      commitAction="Saturation"
      bind:value={editor.edits.basic.saturation}
      min={-100}
      max={100}
    />
  </div>
</div>
