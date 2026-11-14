<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { scopes } from '#lib/stores/scopes.svelte.ts';
  import Histogram from './Histogram.svelte';
  import Vectorscope from './Vectorscope.svelte';
  import Waveform from './Waveform.svelte';

  let { height, mini = false }: { height: number; mini?: boolean } = $props();

  const grid = $derived(scopes.grid);
  const hist = $derived(editor.meta?.histogram ?? null);
  const linearHist = $derived(editor.meta?.linear_histogram ?? null);
</script>

{#snippet placeholder(pending: boolean)}
  <div
    role="status"
    class="flex items-center justify-center text-[10px] {mini
      ? 'text-current opacity-60'
      : 'bg-plot-surface text-plot-ink/65'}"
    style:height="{height}px"
  >
    {pending ? 'Loading…' : 'No data'}
  </div>
{/snippet}

{#if scopes.mode === 'histogram'}
  {#if hist === null}
    {@render placeholder(editor.meta === null)}
  {:else}
    <Histogram {hist} linear={linearHist} gain={scopes.gain} {height} {mini} />
  {/if}
{:else if grid === null}
  {@render placeholder(scopes.loading)}
{:else if scopes.mode === 'vectorscope'}
  <Vectorscope {grid} gain={scopes.gain} zoom={scopes.zoom} size={height} {mini} />
{:else}
  <Waveform
    {grid}
    gain={scopes.gain}
    {height}
    {mini}
    parade={scopes.mode === 'parade'}
    label={scopes.mode === 'parade' ? 'RGB parade' : 'Waveform'}
  />
{/if}
