<script lang="ts">
  import { RadioGroup } from 'bits-ui';
  import Histogram from './scopes/Histogram.svelte';
  import Waveform from './scopes/Waveform.svelte';
  import Vectorscope from './scopes/Vectorscope.svelte';
  import ResizeHandle from '$lib/components/shell/ResizeHandle.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    scopes,
    MAX_SCOPE_HEIGHT,
    MIN_SCOPE_HEIGHT,
    type WaveformChannels
  } from '$lib/stores/scopes.svelte';

  const GAINS = [1, 2, 4, 8];

  const groupClass = 'flex gap-0.5';
  const itemClass =
    'flex h-5 items-center justify-center rounded-sm px-1.5 text-[10px] font-medium text-dark/60 transition-colors hover:text-dark aria-checked:bg-white/8 aria-checked:text-dark';

  const grid = $derived(scopes.grid);
  const hist = $derived(editor.meta?.histogram ?? null);
  const linearHist = $derived(editor.meta?.linear_histogram ?? null);

  function selectChannels(value: string): void {
    scopes.setChannels(value as WaveformChannels);
  }
</script>

{#snippet placeholder(pending: boolean)}
  <div
    role="status"
    class="flex items-center justify-center bg-neutral-950 text-[10px] text-dark/65"
    style:height="{scopes.height}px"
  >
    {pending ? 'Loading…' : 'No data'}
  </div>
{/snippet}

<div class="flex flex-col gap-1">
  <div class="relative">
    {#if scopes.mode === 'histogram'}
      {#if hist === null}
        {@render placeholder(editor.meta === null)}
      {:else}
        <Histogram {hist} linear={linearHist} gain={scopes.gain} height={scopes.height} />
      {/if}
    {:else if grid === null}
      {@render placeholder(scopes.loading)}
    {:else if scopes.mode === 'vectorscope'}
      <Vectorscope {grid} gain={scopes.gain} zoom={scopes.zoom} size={scopes.height} />
    {:else}
      <Waveform
        {grid}
        gain={scopes.gain}
        height={scopes.height}
        parade={scopes.mode === 'parade'}
        label={scopes.mode === 'parade' ? 'RGB parade' : 'Waveform'}
      />
    {/if}
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
      <RadioGroup.Root
        value={scopes.channels}
        onValueChange={selectChannels}
        orientation="horizontal"
        aria-label="Waveform channels"
        class={groupClass}
      >
        <RadioGroup.Item value="luma" class={itemClass}>Luma</RadioGroup.Item>
        <RadioGroup.Item value="rgb" class={itemClass}>RGB</RadioGroup.Item>
      </RadioGroup.Root>
    {/if}

    <RadioGroup.Root
      value={String(scopes.gain)}
      onValueChange={(v) => scopes.setGain(Number(v))}
      orientation="horizontal"
      aria-label="Scope gain"
      class="{groupClass} ml-auto"
    >
      {#each GAINS as gain (gain)}
        <RadioGroup.Item value={String(gain)} class={itemClass}>
          {gain}×
        </RadioGroup.Item>
      {/each}
    </RadioGroup.Root>

    {#if scopes.mode === 'vectorscope'}
      <RadioGroup.Root
        value={String(scopes.zoom)}
        onValueChange={(v) => scopes.setZoom(Number(v))}
        orientation="horizontal"
        aria-label="Vectorscope zoom"
        class={groupClass}
      >
        <RadioGroup.Item value="1" class={itemClass}>1:1</RadioGroup.Item>
        <RadioGroup.Item value="2" class={itemClass}>2:1</RadioGroup.Item>
      </RadioGroup.Root>
    {/if}
  </div>
</div>
