<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import ScopePlot from './scopes/ScopePlot.svelte';
  import ResizeHandle from '#lib/components/shell/ResizeHandle.svelte';
  import {
    scopes,
    MAX_SCOPE_HEIGHT,
    MIN_SCOPE_HEIGHT,
    type WaveformChannels
  } from '#lib/stores/scopes.svelte.ts';

  const GAINS = [1, 2, 4, 8];

  function selectChannels(value: string): void {
    scopes.setChannels(value as WaveformChannels);
  }
</script>

<div class="flex flex-col gap-1">
  <div class="relative">
    <ScopePlot height={scopes.height} />
    <ResizeHandle
      label="Resize scopes"
      orientation="vertical"
      grow="end"
      value={scopes.height}
      min={MIN_SCOPE_HEIGHT}
      max={MAX_SCOPE_HEIGHT}
      step={8}
      shiftStep={32}
      class="absolute inset-x-0 -bottom-1.5 z-10 h-3 cursor-row-resize bg-transparent outline-none after:absolute after:inset-x-0 after:top-1/2 after:h-px after:-translate-y-1/2 after:transition-colors hover:after:bg-primary focus-visible:after:bg-primary"
      activeClass="after:bg-primary"
      onLive={scopes.setHeight}
      onCommit={scopes.commitHeight}
    />
  </div>

  <div class="flex items-center gap-2">
    {#if scopes.mode === 'waveform'}
      <SegmentedChoice
        appearance="mini"
        label="Waveform channels"
        value={scopes.channels}
        options={[
          { value: 'luma', label: 'Luma' },
          { value: 'rgb', label: 'RGB' }
        ]}
        onChange={selectChannels}
      />
    {/if}

    <div class="ml-auto">
      <SegmentedChoice
        appearance="mini"
        label="Scope gain"
        value={String(scopes.gain)}
        options={GAINS.map((gain) => ({ value: String(gain), label: `${gain}×` }))}
        onChange={(v) => scopes.setGain(Number(v))}
      />
    </div>

    {#if scopes.mode === 'vectorscope'}
      <SegmentedChoice
        appearance="mini"
        label="Vectorscope zoom"
        value={String(scopes.zoom)}
        options={[
          { value: '1', label: '1:1' },
          { value: '2', label: '2:1' }
        ]}
        onChange={(v) => scopes.setZoom(Number(v))}
      />
    {/if}
  </div>
</div>
