<script lang="ts">
  import { RadioGroup } from 'bits-ui';
  import Histogram from './scopes/Histogram.svelte';
  import Waveform from './scopes/Waveform.svelte';
  import Vectorscope from './scopes/Vectorscope.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { scopes, type ScopeMode, type WaveformChannels } from '$lib/stores/scopes.svelte';

  const MODES: { value: ScopeMode; label: string }[] = [
    { value: 'histogram', label: 'Hist' },
    { value: 'waveform', label: 'Wave' },
    { value: 'parade', label: 'Parade' },
    { value: 'vectorscope', label: 'Vector' }
  ];
  const GAINS = [1, 2, 4, 8];

  const groupClass = 'flex gap-0.5';
  const itemClass =
    'flex h-5 items-center justify-center rounded-sm px-1.5 text-[10px] font-medium text-dark/60 transition-colors hover:text-dark aria-checked:bg-white/8 aria-checked:text-dark';

  const grid = $derived(scopes.grid);
  const hist = $derived(editor.meta?.histogram ?? null);
  const linearHist = $derived(editor.meta?.linear_histogram ?? null);

  function selectMode(value: string): void {
    scopes.setMode(value as ScopeMode);
    if (scopes.needsRender) editor.refreshScopes();
  }

  function selectChannels(value: string): void {
    scopes.setChannels(value as WaveformChannels);
  }
</script>

{#snippet placeholder(pending: boolean)}
  <div
    role="status"
    class="flex h-32 items-center justify-center bg-neutral-950 text-[10px] text-dark/65"
  >
    {pending ? 'Loading…' : 'No data'}
  </div>
{/snippet}

<div class="flex flex-col gap-1">
  <RadioGroup.Root
    value={scopes.mode}
    onValueChange={selectMode}
    orientation="horizontal"
    aria-label="Scope"
    class="{groupClass} justify-between"
  >
    {#each MODES as mode (mode.value)}
      <RadioGroup.Item value={mode.value} class="{itemClass} flex-1">
        {mode.label}
      </RadioGroup.Item>
    {/each}
  </RadioGroup.Root>

  {#if scopes.mode === 'histogram'}
    {#if hist === null}
      {@render placeholder(editor.meta === null)}
    {:else}
      <Histogram {hist} linear={linearHist} gain={scopes.gain} />
    {/if}
  {:else if grid === null}
    {@render placeholder(scopes.loading)}
  {:else if scopes.mode === 'vectorscope'}
    <Vectorscope {grid} gain={scopes.gain} zoom={scopes.zoom} />
  {:else}
    <Waveform
      {grid}
      gain={scopes.gain}
      parade={scopes.mode === 'parade'}
      label={scopes.mode === 'parade' ? 'RGB parade' : 'Waveform'}
    />
  {/if}

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
