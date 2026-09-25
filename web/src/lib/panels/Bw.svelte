<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import HueWheel from '$lib/components/editor/controls/HueWheel.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import { editsToManifest } from '$lib/edits/manifest';
  import { editor } from '$lib/stores/editor.svelte';
  import { BW_CHANNELS, neutralBw, type BwChannel } from '$lib/types/edits';

  const CHANNEL_HUES: Record<BwChannel, number> = {
    red: 0,
    yellow: 60,
    green: 120,
    aqua: 180,
    blue: 240,
    magenta: 300
  };

  const TINTS = [
    { key: 'shadows', label: 'Shadows' },
    { key: 'highlights', label: 'Highlights' }
  ] as const;

  const bw = $derived(editor.edits.color.bw);
  const modified = $derived('bw' in editsToManifest(editor.edits).ops);

  function channelLabel(channel: BwChannel): string {
    return channel.charAt(0).toUpperCase() + channel.slice(1);
  }

  function channelGradient(channel: BwChannel): string {
    const hue = CHANNEL_HUES[channel];
    return `linear-gradient(to right, hsl(${hue}, 0%, 15%), hsl(${hue}, 70%, 55%), hsl(${hue}, 0%, 90%))`;
  }

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
      {#each BW_CHANNELS as channel (channel)}
        <EditSlider
          label={channelLabel(channel)}
          commitAction={`B&W ${channelLabel(channel)}`}
          bind:value={editor.edits.color.bw.mix[channel]}
          min={-100}
          max={100}
          gradient={channelGradient(channel)}
        />
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
      <EditSlider
        label="Balance"
        commitAction="B&W Tint Balance"
        bind:value={editor.edits.color.bw.balance}
        min={-100}
        max={100}
        gradient="var(--gradient-grading-balance)"
      />
    </div>
  {/if}
</div>
