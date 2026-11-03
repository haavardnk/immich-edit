<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import HueWheel from '#lib/components/editor/controls/HueWheel.svelte';
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import { editsToManifest } from '#lib/edits/manifest.ts';
  import { BW_MIX, bwBalance } from '#lib/editor/params/color.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { neutralBw } from '#lib/types/edits.ts';

  const TINTS = [
    { key: 'shadows', label: 'Shadows' },
    { key: 'highlights', label: 'Highlights' }
  ] as const;

  const bw = $derived(editor.edits.color.bw);
  const modified = $derived('bw' in editsToManifest(editor.edits).ops);

  function toggle(enabled: boolean): void {
    editor.edits.color.bw.enabled = enabled;
    void editor.onCommit(enabled ? 'Black & White' : 'Color');
  }

  function reset(): void {
    editor.edits.color.bw = neutralBw();
    void editor.onCommit('Reset Black & White');
  }
</script>

<div class="flex flex-col gap-1.5 pb-1">
  <SectionHeader title="B&W" section="bw" {modified} onReset={reset} />
  <CheckboxRow label="Convert to Black & White" checked={bw.enabled} onChange={toggle} />

  {#if bw.enabled}
    <div class="flex flex-col gap-1 border-t border-hairline pt-1">
      {#each BW_MIX as param (param.id)}
        <ParamSlider {param} />
      {/each}
    </div>

    <div class="flex flex-col gap-1 border-t border-hairline pt-1">
      <div class="grid grid-cols-2 justify-items-center gap-x-2">
        {#each TINTS as tint (tint.key)}
          <div class="flex flex-col items-center gap-0.5">
            <HueWheel
              bind:hue={editor.edits.color.bw[tint.key].hue}
              bind:sat={editor.edits.color.bw[tint.key].sat}
              size={78}
              onLive={editor.onLive}
              onCommit={editor.onCommit}
              commitAction="{tint.label} Tint"
            />
            <span class="text-[10px] font-medium text-dark/65">{tint.label}</span>
          </div>
        {/each}
      </div>
      <ParamSlider param={bwBalance} />
    </div>
  {/if}
</div>
